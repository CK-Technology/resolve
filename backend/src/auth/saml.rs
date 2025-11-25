//! SAML 2.0 authentication support for enterprise SSO
//!
//! Implements SAML 2.0 Service Provider (SP) functionality for:
//! - Azure AD / ADFS
//! - Okta
//! - OneLogin
//! - Generic SAML 2.0 IdPs

use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, ApiResult};

/// SAML 2.0 Service Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlSpConfig {
    /// Unique entity ID for this SP (usually the app URL)
    pub entity_id: String,
    /// Assertion Consumer Service URL (where IdP sends responses)
    pub acs_url: String,
    /// Single Logout URL (optional)
    pub slo_url: Option<String>,
    /// SP signing certificate (PEM format)
    pub signing_cert: Option<String>,
    /// SP signing private key (PEM format, encrypted)
    pub signing_key: Option<String>,
    /// Whether to sign AuthnRequests
    pub sign_requests: bool,
    /// Whether assertions must be signed
    pub require_signed_assertions: bool,
    /// Whether assertions must be encrypted
    pub require_encrypted_assertions: bool,
    /// NameID format preference
    pub name_id_format: NameIdFormat,
}

/// SAML 2.0 Identity Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlIdpConfig {
    pub provider_id: Uuid,
    pub name: String,
    pub display_name: String,
    /// IdP Entity ID
    pub entity_id: String,
    /// Single Sign-On URL (HTTP-Redirect or HTTP-POST)
    pub sso_url: String,
    /// SSO binding type
    pub sso_binding: SamlBinding,
    /// Single Logout URL (optional)
    pub slo_url: Option<String>,
    /// IdP signing certificate (PEM format) - for verifying signatures
    pub signing_cert: String,
    /// Optional encryption certificate
    pub encryption_cert: Option<String>,
    /// Attribute mappings
    pub attribute_mapping: SamlAttributeMapping,
    pub enabled: bool,
    /// Allow new user registration via this IdP
    pub allow_registration: bool,
    /// Restrict to specific email domains
    pub allowed_domains: Vec<String>,
    /// Map IdP groups/roles to Resolve roles
    pub role_mapping: Option<SamlRoleMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SamlBinding {
    HttpRedirect,
    HttpPost,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NameIdFormat {
    /// urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress
    EmailAddress,
    /// urn:oasis:names:tc:SAML:2.0:nameid-format:persistent
    Persistent,
    /// urn:oasis:names:tc:SAML:2.0:nameid-format:transient
    Transient,
    /// urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified
    Unspecified,
}

impl NameIdFormat {
    pub fn as_urn(&self) -> &'static str {
        match self {
            Self::EmailAddress => "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress",
            Self::Persistent => "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent",
            Self::Transient => "urn:oasis:names:tc:SAML:2.0:nameid-format:transient",
            Self::Unspecified => "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified",
        }
    }
}

/// Mapping of SAML attributes to user fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAttributeMapping {
    /// Attribute name for email
    pub email: String,
    /// Attribute name for first name
    pub first_name: Option<String>,
    /// Attribute name for last name
    pub last_name: Option<String>,
    /// Attribute name for display name (fallback if first/last not available)
    pub display_name: Option<String>,
    /// Attribute name for groups/roles
    pub groups: Option<String>,
}

impl Default for SamlAttributeMapping {
    fn default() -> Self {
        Self {
            email: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress".to_string(),
            first_name: Some(
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname".to_string(),
            ),
            last_name: Some(
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname".to_string(),
            ),
            display_name: Some(
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name".to_string(),
            ),
            groups: Some(
                "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups".to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlRoleMapping {
    /// IdP group/role name -> Resolve role ID
    pub group_to_role: HashMap<String, Uuid>,
    /// Default role if no mapping matches
    pub default_role_id: Option<Uuid>,
}

/// SAML authentication state stored during auth flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAuthState {
    pub provider_id: Uuid,
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    /// Where to redirect after successful auth
    pub return_url: Option<String>,
}

/// User info extracted from SAML assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlUserInfo {
    /// NameID from the assertion
    pub name_id: String,
    /// NameID format
    pub name_id_format: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    /// Session index for SLO
    pub session_index: Option<String>,
    /// All attributes from assertion
    pub attributes: HashMap<String, Vec<String>>,
}

/// Generate SAML AuthnRequest
pub fn generate_authn_request(
    sp_config: &SamlSpConfig,
    idp_config: &SamlIdpConfig,
    return_url: Option<String>,
) -> ApiResult<(String, SamlAuthState)> {
    let request_id = format!("_resolve_{}", Uuid::new_v4().to_string().replace("-", ""));
    let issue_instant = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Build AuthnRequest XML
    let authn_request = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest
    xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
    xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
    ID="{request_id}"
    Version="2.0"
    IssueInstant="{issue_instant}"
    Destination="{destination}"
    AssertionConsumerServiceURL="{acs_url}"
    ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST">
    <saml:Issuer>{entity_id}</saml:Issuer>
    <samlp:NameIDPolicy
        Format="{name_id_format}"
        AllowCreate="true"/>
</samlp:AuthnRequest>"#,
        request_id = request_id,
        issue_instant = issue_instant,
        destination = idp_config.sso_url,
        acs_url = sp_config.acs_url,
        entity_id = sp_config.entity_id,
        name_id_format = sp_config.name_id_format.as_urn(),
    );

    // Build redirect URL based on binding
    let auth_url = match idp_config.sso_binding {
        SamlBinding::HttpRedirect => {
            // Deflate and base64 encode for redirect binding
            let compressed = deflate_compress(authn_request.as_bytes())?;
            let encoded = general_purpose::STANDARD.encode(&compressed);
            let url_encoded = urlencoding::encode(&encoded);
            format!("{}?SAMLRequest={}", idp_config.sso_url, url_encoded)
        }
        SamlBinding::HttpPost => {
            // For POST binding, we return a form that the frontend should auto-submit
            let encoded = general_purpose::STANDARD.encode(authn_request.as_bytes());
            // Return the SSO URL - frontend will POST the SAMLRequest
            format!(
                "{}#SAMLRequest={}",
                idp_config.sso_url,
                urlencoding::encode(&encoded)
            )
        }
    };

    let state = SamlAuthState {
        provider_id: idp_config.provider_id,
        request_id,
        created_at: Utc::now(),
        return_url,
    };

    Ok((auth_url, state))
}

/// Parse and validate SAML Response
pub fn parse_saml_response(
    sp_config: &SamlSpConfig,
    idp_config: &SamlIdpConfig,
    saml_response: &str,
    state: &SamlAuthState,
) -> ApiResult<SamlUserInfo> {
    // Decode the response
    let decoded = general_purpose::STANDARD
        .decode(saml_response)
        .map_err(|e| AppError::OAuthError(format!("Invalid SAML response encoding: {}", e)))?;

    let xml = String::from_utf8(decoded)
        .map_err(|e| AppError::OAuthError(format!("Invalid SAML response encoding: {}", e)))?;

    // Parse XML and validate signature
    // Note: In production, use the `samael` crate for proper SAML handling
    // This is a simplified implementation for structure demonstration

    // Validate InResponseTo matches our request ID
    if !xml.contains(&format!("InResponseTo=\"{}\"", state.request_id)) {
        return Err(AppError::OAuthError(
            "SAML response does not match request".to_string(),
        ));
    }

    // Validate the response is not too old (5 minute window)
    let max_age = chrono::Duration::minutes(5);
    if Utc::now() - state.created_at > max_age {
        return Err(AppError::OAuthError("SAML request expired".to_string()));
    }

    // Extract user info from assertion
    // In production, properly parse XML and verify signatures using samael
    let user_info = extract_user_from_xml(&xml, &idp_config.attribute_mapping)?;

    // Validate email domain if restrictions are configured
    if !idp_config.allowed_domains.is_empty() {
        let email_domain = user_info
            .email
            .split('@')
            .nth(1)
            .ok_or_else(|| AppError::OAuthError("Invalid email format".to_string()))?;

        if !idp_config.allowed_domains.iter().any(|d| d == email_domain) {
            return Err(AppError::Forbidden(format!(
                "Email domain '{}' is not allowed for this provider",
                email_domain
            )));
        }
    }

    Ok(user_info)
}

/// Generate SP metadata XML for IdP configuration
pub fn generate_sp_metadata(sp_config: &SamlSpConfig) -> String {
    let mut metadata = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor
    xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
    entityID="{entity_id}">
    <md:SPSSODescriptor
        AuthnRequestsSigned="{sign_requests}"
        WantAssertionsSigned="{require_signed}"
        protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
        <md:NameIDFormat>{name_id_format}</md:NameIDFormat>
        <md:AssertionConsumerService
            Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
            Location="{acs_url}"
            index="0"
            isDefault="true"/>"#,
        entity_id = sp_config.entity_id,
        sign_requests = sp_config.sign_requests,
        require_signed = sp_config.require_signed_assertions,
        name_id_format = sp_config.name_id_format.as_urn(),
        acs_url = sp_config.acs_url,
    );

    if let Some(slo_url) = &sp_config.slo_url {
        metadata.push_str(&format!(
            r#"
        <md:SingleLogoutService
            Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"
            Location="{slo_url}"/>"#,
            slo_url = slo_url
        ));
    }

    if let Some(cert) = &sp_config.signing_cert {
        // Extract just the certificate data (remove PEM headers)
        let cert_data = cert
            .lines()
            .filter(|l| !l.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");

        metadata.push_str(&format!(
            r#"
        <md:KeyDescriptor use="signing">
            <ds:KeyInfo xmlns:ds="http://www.w3.org/2000/09/xmldsig#">
                <ds:X509Data>
                    <ds:X509Certificate>{cert}</ds:X509Certificate>
                </ds:X509Data>
            </ds:KeyInfo>
        </md:KeyDescriptor>"#,
            cert = cert_data
        ));
    }

    metadata.push_str(
        r#"
    </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
    );

    metadata
}

// Helper: Simple XML parsing for user info (in production, use proper XML/SAML library)
fn extract_user_from_xml(
    xml: &str,
    mapping: &SamlAttributeMapping,
) -> ApiResult<SamlUserInfo> {
    // This is a simplified parser - production should use samael or similar
    let mut attributes: HashMap<String, Vec<String>> = HashMap::new();

    // Extract NameID
    let name_id = extract_xml_value(xml, "NameID")
        .ok_or_else(|| AppError::OAuthError("Missing NameID in SAML assertion".to_string()))?;

    // Extract session index if present
    let session_index = extract_xml_attribute(xml, "AuthnStatement", "SessionIndex");

    // For demo: extract some common attributes
    // In production, properly parse Attribute elements
    let email = extract_attribute_value(xml, &mapping.email)
        .or_else(|| {
            // Fallback: use NameID if it looks like an email
            if name_id.contains('@') {
                Some(name_id.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| AppError::OAuthError("Missing email in SAML assertion".to_string()))?;

    let first_name = mapping
        .first_name
        .as_ref()
        .and_then(|attr| extract_attribute_value(xml, attr));

    let last_name = mapping
        .last_name
        .as_ref()
        .and_then(|attr| extract_attribute_value(xml, attr));

    let display_name = mapping
        .display_name
        .as_ref()
        .and_then(|attr| extract_attribute_value(xml, attr));

    let groups = mapping
        .groups
        .as_ref()
        .map(|attr| extract_attribute_values(xml, attr))
        .unwrap_or_default();

    // Store all found attributes
    attributes.insert(mapping.email.clone(), vec![email.clone()]);
    if let Some(ref fn_val) = first_name {
        if let Some(ref attr_name) = mapping.first_name {
            attributes.insert(attr_name.clone(), vec![fn_val.clone()]);
        }
    }

    Ok(SamlUserInfo {
        name_id,
        name_id_format: "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress".to_string(),
        email,
        first_name,
        last_name,
        display_name,
        groups,
        session_index,
        attributes,
    })
}

// Helper functions for simple XML extraction
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}[^>]*>", tag);
    let end_tag = format!("</{}>", tag);

    if let Ok(re) = regex::Regex::new(&format!("{}([^<]*){}", start_tag, end_tag)) {
        if let Some(caps) = re.captures(xml) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

fn extract_xml_attribute(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"<{}[^>]*{}="([^"]*)"[^>]*>"#, tag, attr);
    if let Ok(re) = regex::Regex::new(&pattern) {
        if let Some(caps) = re.captures(xml) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

fn extract_attribute_value(xml: &str, attribute_name: &str) -> Option<String> {
    let pattern = format!(
        r#"<(?:saml:|saml2:)?Attribute[^>]*Name="{}[^>]*>.*?<(?:saml:|saml2:)?AttributeValue[^>]*>([^<]*)<"#,
        regex::escape(attribute_name)
    );
    if let Ok(re) = regex::Regex::new(&pattern) {
        if let Some(caps) = re.captures(xml) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

fn extract_attribute_values(xml: &str, attribute_name: &str) -> Vec<String> {
    let mut values = Vec::new();
    let pattern = format!(
        r#"<(?:saml:|saml2:)?Attribute[^>]*Name="{}[^>]*>(.*?)</(?:saml:|saml2:)?Attribute>"#,
        regex::escape(attribute_name)
    );

    if let Ok(re) = regex::Regex::new(&pattern) {
        if let Some(caps) = re.captures(xml) {
            if let Some(attr_content) = caps.get(1) {
                let value_pattern = r#"<(?:saml:|saml2:)?AttributeValue[^>]*>([^<]*)<"#;
                if let Ok(value_re) = regex::Regex::new(value_pattern) {
                    for cap in value_re.captures_iter(attr_content.as_str()) {
                        if let Some(val) = cap.get(1) {
                            values.push(val.as_str().to_string());
                        }
                    }
                }
            }
        }
    }
    values
}

// Deflate compression for SAML HTTP-Redirect binding
fn deflate_compress(data: &[u8]) -> ApiResult<Vec<u8>> {
    use flate2::write::DeflateEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| AppError::InternalError(format!("Compression failed: {}", e)))?;
    encoder
        .finish()
        .map_err(|e| AppError::InternalError(format!("Compression failed: {}", e)))
}

/// Pre-configured IdP templates for common providers
pub mod templates {
    use super::*;

    pub fn azure_ad_template(tenant_id: &str) -> SamlIdpConfig {
        SamlIdpConfig {
            provider_id: Uuid::new_v4(),
            name: "azure_ad".to_string(),
            display_name: "Azure AD (SAML)".to_string(),
            entity_id: format!("https://sts.windows.net/{}/", tenant_id),
            sso_url: format!(
                "https://login.microsoftonline.com/{}/saml2",
                tenant_id
            ),
            sso_binding: SamlBinding::HttpRedirect,
            slo_url: Some(format!(
                "https://login.microsoftonline.com/{}/saml2",
                tenant_id
            )),
            signing_cert: String::new(), // Must be filled from Azure AD metadata
            encryption_cert: None,
            attribute_mapping: SamlAttributeMapping::default(),
            enabled: true,
            allow_registration: true,
            allowed_domains: vec![],
            role_mapping: None,
        }
    }

    pub fn okta_template(domain: &str) -> SamlIdpConfig {
        SamlIdpConfig {
            provider_id: Uuid::new_v4(),
            name: "okta".to_string(),
            display_name: "Okta".to_string(),
            entity_id: format!("http://www.okta.com/{}", domain),
            sso_url: format!("https://{}.okta.com/app/resolve/sso/saml", domain),
            sso_binding: SamlBinding::HttpPost,
            slo_url: None,
            signing_cert: String::new(), // Must be filled from Okta metadata
            encryption_cert: None,
            attribute_mapping: SamlAttributeMapping {
                email: "email".to_string(),
                first_name: Some("firstName".to_string()),
                last_name: Some("lastName".to_string()),
                display_name: Some("displayName".to_string()),
                groups: Some("groups".to_string()),
            },
            enabled: true,
            allow_registration: true,
            allowed_domains: vec![],
            role_mapping: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_id_format() {
        assert_eq!(
            NameIdFormat::EmailAddress.as_urn(),
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
        );
    }

    #[test]
    fn test_sp_metadata_generation() {
        let config = SamlSpConfig {
            entity_id: "https://resolve.example.com".to_string(),
            acs_url: "https://resolve.example.com/api/v1/auth/saml/callback".to_string(),
            slo_url: None,
            signing_cert: None,
            signing_key: None,
            sign_requests: false,
            require_signed_assertions: true,
            require_encrypted_assertions: false,
            name_id_format: NameIdFormat::EmailAddress,
        };

        let metadata = generate_sp_metadata(&config);
        assert!(metadata.contains("entityID=\"https://resolve.example.com\""));
        assert!(metadata.contains("AssertionConsumerService"));
    }
}
