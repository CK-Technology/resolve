use crate::models::passwords::*;
use crate::services::encryption::EncryptionService;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use regex::Regex;
use serde_json;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PasswordManagerService {
    db_pool: PgPool,
    encryption_service: EncryptionService,
    phonetic_map: HashMap<char, &'static str>,
}

impl PasswordManagerService {
    pub fn new(db_pool: PgPool, encryption_service: EncryptionService) -> Self {
        let mut phonetic_map = HashMap::new();
        
        // NATO Phonetic Alphabet for letters
        phonetic_map.insert('A', "Alpha");
        phonetic_map.insert('B', "Bravo");
        phonetic_map.insert('C', "Charlie");
        phonetic_map.insert('D', "Delta");
        phonetic_map.insert('E', "Echo");
        phonetic_map.insert('F', "Foxtrot");
        phonetic_map.insert('G', "Golf");
        phonetic_map.insert('H', "Hotel");
        phonetic_map.insert('I', "India");
        phonetic_map.insert('J', "Juliet");
        phonetic_map.insert('K', "Kilo");
        phonetic_map.insert('L', "Lima");
        phonetic_map.insert('M', "Mike");
        phonetic_map.insert('N', "November");
        phonetic_map.insert('O', "Oscar");
        phonetic_map.insert('P', "Papa");
        phonetic_map.insert('Q', "Quebec");
        phonetic_map.insert('R', "Romeo");
        phonetic_map.insert('S', "Sierra");
        phonetic_map.insert('T', "Tango");
        phonetic_map.insert('U', "Uniform");
        phonetic_map.insert('V', "Victor");
        phonetic_map.insert('W', "Whiskey");
        phonetic_map.insert('X', "X-ray");
        phonetic_map.insert('Y', "Yankee");
        phonetic_map.insert('Z', "Zulu");

        // Numbers
        phonetic_map.insert('0', "Zero");
        phonetic_map.insert('1', "One");
        phonetic_map.insert('2', "Two");
        phonetic_map.insert('3', "Three");
        phonetic_map.insert('4', "Four");
        phonetic_map.insert('5', "Five");
        phonetic_map.insert('6', "Six");
        phonetic_map.insert('7', "Seven");
        phonetic_map.insert('8', "Eight");
        phonetic_map.insert('9', "Nine");

        // Common symbols
        phonetic_map.insert('!', "Exclamation");
        phonetic_map.insert('@', "At-sign");
        phonetic_map.insert('#', "Hash");
        phonetic_map.insert('$', "Dollar");
        phonetic_map.insert('%', "Percent");
        phonetic_map.insert('^', "Caret");
        phonetic_map.insert('&', "Ampersand");
        phonetic_map.insert('*', "Asterisk");
        phonetic_map.insert('(', "Open-paren");
        phonetic_map.insert(')', "Close-paren");
        phonetic_map.insert('-', "Dash");
        phonetic_map.insert('_', "Underscore");
        phonetic_map.insert('+', "Plus");
        phonetic_map.insert('=', "Equals");
        phonetic_map.insert('[', "Open-bracket");
        phonetic_map.insert(']', "Close-bracket");
        phonetic_map.insert('{', "Open-brace");
        phonetic_map.insert('}', "Close-brace");
        phonetic_map.insert('|', "Pipe");
        phonetic_map.insert('\\', "Backslash");
        phonetic_map.insert(':', "Colon");
        phonetic_map.insert(';', "Semicolon");
        phonetic_map.insert('"', "Quote");
        phonetic_map.insert('\'', "Apostrophe");
        phonetic_map.insert('<', "Less-than");
        phonetic_map.insert('>', "Greater-than");
        phonetic_map.insert(',', "Comma");
        phonetic_map.insert('.', "Period");
        phonetic_map.insert('?', "Question");
        phonetic_map.insert('/', "Slash");
        phonetic_map.insert('~', "Tilde");
        phonetic_map.insert('`', "Backtick");

        Self {
            db_pool,
            encryption_service,
            phonetic_map,
        }
    }

    pub async fn create_password(&self, request: CreatePasswordRequest, created_by: Uuid) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let password_id = Uuid::new_v4();
        
        // Encrypt password and notes
        let encrypted_password = self.encryption_service.encrypt(&request.password)?;
        let encrypted_notes = if let Some(notes) = &request.notes {
            Some(self.encryption_service.encrypt(notes)?)
        } else {
            None
        };
        
        // Encrypt OTP secret if provided
        let encrypted_otp_secret = if let Some(otp_secret) = &request.otp_secret {
            Some(self.encryption_service.encrypt(otp_secret)?)
        } else {
            None
        };

        // Calculate password strength
        let strength_score = self.calculate_password_strength(&request.password);

        // Serialize tags
        let tags_json = serde_json::to_string(&request.tags)?;

        sqlx::query!(
            r#"
            INSERT INTO passwords (id, client_id, name, description, username, password_encrypted,
                                 url, notes_encrypted, category, tags, favorite, otp_secret_encrypted,
                                 phonetic_enabled, created_by, created_at, updated_at, expires_at,
                                 strength_score, breach_detected, folder_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, false, $11, $12, $13, NOW(), NOW(), $14, $15, false, $16)
            "#,
            password_id,
            request.client_id,
            request.name,
            request.description,
            request.username,
            encrypted_password,
            request.url,
            encrypted_notes,
            request.category,
            tags_json,
            encrypted_otp_secret,
            request.phonetic_enabled,
            created_by,
            request.expires_at,
            strength_score,
            request.folder_id
        )
        .execute(&self.db_pool)
        .await?;

        info!("Created password '{}' with ID: {}", request.name, password_id);
        Ok(password_id)
    }

    pub async fn get_password(&self, id: Uuid, user_id: Uuid) -> Result<Option<PasswordResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let result = sqlx::query!(
            r#"
            SELECT p.*, c.name as client_name, u.name as created_by_name,
                   f.name as folder_name
            FROM passwords p
            LEFT JOIN clients c ON p.client_id = c.id
            LEFT JOIN users u ON p.created_by = u.id
            LEFT JOIN password_folders f ON p.folder_id = f.id
            WHERE p.id = $1
            "#,
            id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = result {
            // Decrypt password and notes
            let decrypted_password = self.encryption_service.decrypt(&row.password_encrypted)?;
            let decrypted_notes = if let Some(encrypted_notes) = row.notes_encrypted {
                Some(self.encryption_service.decrypt(&encrypted_notes)?)
            } else {
                None
            };

            // Generate phonetic password if enabled
            let phonetic_password = if row.phonetic_enabled {
                Some(self.generate_phonetic_representation(&decrypted_password))
            } else {
                None
            };

            // Generate OTP code if available
            let (has_otp, otp_code) = if let Some(encrypted_otp_secret) = row.otp_secret_encrypted {
                let otp_secret = self.encryption_service.decrypt(&encrypted_otp_secret)?;
                let otp_code = self.generate_totp(&otp_secret)?;
                (true, Some(otp_code))
            } else {
                (false, None)
            };

            // Parse tags
            let tags: Vec<String> = serde_json::from_str(&row.tags).unwrap_or_default();

            // Check if expires soon (within 30 days)
            let expires_soon = if let Some(expires_at) = row.expires_at {
                expires_at <= Utc::now() + Duration::days(30)
            } else {
                false
            };

            // Update last accessed timestamp
            sqlx::query!(
                "UPDATE passwords SET last_accessed = NOW() WHERE id = $1",
                id
            )
            .execute(&self.db_pool)
            .await?;

            Ok(Some(PasswordResponse {
                id: row.id,
                client_id: row.client_id,
                client_name: row.client_name,
                name: row.name,
                description: row.description,
                username: row.username,
                password: decrypted_password,
                phonetic_password,
                url: row.url,
                notes: decrypted_notes,
                category: row.category,
                tags,
                favorite: row.favorite,
                has_otp,
                otp_code,
                phonetic_enabled: row.phonetic_enabled,
                created_by: row.created_by,
                created_by_name: row.created_by_name.unwrap_or_else(|| "Unknown".to_string()),
                created_at: row.created_at,
                updated_at: row.updated_at,
                last_accessed: row.last_accessed,
                expires_at: row.expires_at,
                expires_soon,
                strength_score: row.strength_score,
                strength_label: self.get_strength_label(row.strength_score),
                breach_detected: row.breach_detected,
                folder_id: row.folder_id,
                folder_name: row.folder_name,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn generate_phonetic_representation(&self, password: &str) -> String {
        let mut phonetic_parts = Vec::new();
        
        for ch in password.chars() {
            let phonetic = if let Some(phonetic_word) = self.phonetic_map.get(&ch.to_uppercase().next().unwrap_or(ch)) {
                if ch.is_lowercase() {
                    format!("{}-lowercase", phonetic_word)
                } else {
                    phonetic_word.to_string()
                }
            } else {
                format!("'{}'", ch)
            };
            phonetic_parts.push(phonetic);
        }
        
        phonetic_parts.join(" ")
    }

    pub async fn generate_password(&self, request: GeneratePasswordRequest) -> Result<GeneratePasswordResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut charset = String::new();
        
        if request.include_lowercase {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
        }
        if request.include_uppercase {
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        if request.include_numbers {
            charset.push_str("0123456789");
        }
        if request.include_symbols {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }
        
        if request.exclude_ambiguous {
            charset = charset.replace(&['0', 'O', 'l', '1', 'I'][..], "");
        }
        
        if charset.is_empty() {
            return Err("No character sets selected for password generation".into());
        }
        
        let mut rng = rand::thread_rng();
        let password: String = (0..request.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset.chars().nth(idx).unwrap()
            })
            .collect();
        
        let phonetic_password = if request.phonetic_enabled {
            Some(self.generate_phonetic_representation(&password))
        } else {
            None
        };
        
        let strength_score = self.calculate_password_strength(&password);
        
        Ok(GeneratePasswordResponse {
            password,
            phonetic_password,
            strength_score,
            strength_label: self.get_strength_label(strength_score),
        })
    }

    fn calculate_password_strength(&self, password: &str) -> i32 {
        let mut score = 0;
        
        // Base score from length
        score += password.len() as i32 * 4;
        
        // Character diversity bonuses
        if password.chars().any(|c| c.is_lowercase()) {
            score += 5;
        }
        if password.chars().any(|c| c.is_uppercase()) {
            score += 5;
        }
        if password.chars().any(|c| c.is_numeric()) {
            score += 10;
        }
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 15;
        }
        
        // Length bonuses
        if password.len() >= 8 {
            score += 10;
        }
        if password.len() >= 12 {
            score += 20;
        }
        if password.len() >= 16 {
            score += 30;
        }
        
        // Deduct for common patterns
        if self.has_sequential_chars(password) {
            score -= 15;
        }
        if self.has_repeated_chars(password) {
            score -= 10;
        }
        
        score.max(0).min(100)
    }

    fn has_sequential_chars(&self, password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let a = chars[i] as u32;
            let b = chars[i + 1] as u32;
            let c = chars[i + 2] as u32;
            
            if (b == a + 1 && c == b + 1) || (b == a - 1 && c == b - 1) {
                return true;
            }
        }
        false
    }

    fn has_repeated_chars(&self, password: &str) -> bool {
        let mut char_counts = HashMap::new();
        for ch in password.chars() {
            *char_counts.entry(ch).or_insert(0) += 1;
        }
        
        char_counts.values().any(|&count| count >= 3)
    }

    fn get_strength_label(&self, score: i32) -> String {
        match score {
            0..=30 => "Very Weak".to_string(),
            31..=50 => "Weak".to_string(),
            51..=70 => "Fair".to_string(),
            71..=85 => "Good".to_string(),
            86..=95 => "Strong".to_string(),
            _ => "Very Strong".to_string(),
        }
    }

    fn generate_totp(&self, secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let time_step = 30;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() / time_step;
        
        let decoded_secret = base64::decode(secret)?;
        let mut hmac = <Hmac<Sha1> as hmac::Mac>::new_from_slice(&decoded_secret)?;
        hmac.update(&current_time.to_be_bytes());
        let result = hmac.finalize().into_bytes();
        
        let offset = (result[19] & 0x0f) as usize;
        let code = u32::from_be_bytes([
            result[offset] & 0x7f,
            result[offset + 1],
            result[offset + 2],
            result[offset + 3],
        ]) % 1_000_000;
        
        Ok(format!("{:06}", code))
    }

    pub async fn create_folder(&self, request: CreateFolderRequest, created_by: Uuid) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let folder_id = Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO password_folders (id, client_id, name, description, parent_id, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#,
            folder_id,
            request.client_id,
            request.name,
            request.description,
            request.parent_id,
            created_by
        )
        .execute(&self.db_pool)
        .await?;

        Ok(folder_id)
    }

    pub async fn create_password_share(&self, request: CreatePasswordShareRequest, created_by: Uuid, base_url: &str) -> Result<PasswordShareResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Verify password exists and user has access
        let password = sqlx::query!(
            "SELECT name FROM passwords WHERE id = $1",
            request.password_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        let share_id = Uuid::new_v4();
        let share_token = self.generate_secure_token();
        let expires_at = Utc::now() + chrono::Duration::hours(request.expires_in_hours as i64);
        
        let access_password_hash = if let Some(password) = &request.access_password {
            Some(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
        } else {
            None
        };

        sqlx::query!(
            r#"
            INSERT INTO password_shares (id, password_id, share_token, created_by, recipient_email,
                                       recipient_name, expires_at, max_views, view_count,
                                       require_email_verification, require_password, access_password,
                                       one_time_use, created_at, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, $9, $10, $11, $12, NOW(), true)
            "#,
            share_id,
            request.password_id,
            share_token,
            created_by,
            request.recipient_email,
            request.recipient_name,
            expires_at,
            request.max_views,
            request.require_email_verification,
            request.require_password,
            access_password_hash,
            request.one_time_use
        )
        .execute(&self.db_pool)
        .await?;

        // Get created by name
        let created_by_name = sqlx::query!("SELECT name FROM users WHERE id = $1", created_by)
            .fetch_one(&self.db_pool)
            .await?
            .name;

        let share_url = format!("{}/shared-password/{}", base_url, share_token);

        // Send email if recipient email provided
        if let Some(recipient_email) = &request.recipient_email {
            // TODO: Send email notification about password share
            info!("Password share created for {}", recipient_email);
        }

        Ok(PasswordShareResponse {
            id: share_id,
            password_id: request.password_id,
            password_name: password.name,
            share_token,
            share_url,
            recipient_email: request.recipient_email,
            recipient_name: request.recipient_name,
            expires_at,
            max_views: request.max_views,
            view_count: 0,
            require_email_verification: request.require_email_verification,
            require_password: request.require_password,
            one_time_use: request.one_time_use,
            created_at: Utc::now(),
            last_accessed: None,
            is_active: true,
            is_expired: false,
            created_by,
            created_by_name,
        })
    }

    pub async fn access_shared_password(&self, request: AccessPasswordShareRequest) -> Result<Option<PasswordShareAccessResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let share = sqlx::query!(
            r#"
            SELECT ps.*, p.name as password_name, p.password_encrypted, p.username, p.url,
                   p.notes_encrypted, p.otp_secret_encrypted, p.phonetic_enabled
            FROM password_shares ps
            JOIN passwords p ON ps.password_id = p.id
            WHERE ps.share_token = $1 AND ps.is_active = true
            "#,
            request.share_token
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(share) = share {
            // Check if expired
            if share.expires_at <= Utc::now() {
                // Deactivate expired share
                sqlx::query!(
                    "UPDATE password_shares SET is_active = false WHERE id = $1",
                    share.id
                )
                .execute(&self.db_pool)
                .await?;
                return Ok(None);
            }

            // Check view limits
            if let Some(max_views) = share.max_views {
                if share.view_count >= max_views {
                    return Ok(None);
                }
            }

            // Verify access password if required
            if share.require_password {
                if let (Some(provided_password), Some(stored_hash)) = (&request.access_password, &share.access_password) {
                    if !bcrypt::verify(provided_password, stored_hash)? {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }
            }

            // TODO: Verify email verification code if required
            if share.require_email_verification && request.email_verification_code.is_none() {
                return Ok(None);
            }

            // Decrypt password data
            let decrypted_password = self.encryption_service.decrypt(&share.password_encrypted)?;
            let decrypted_notes = if let Some(encrypted_notes) = share.notes_encrypted {
                Some(self.encryption_service.decrypt(&encrypted_notes)?)
            } else {
                None
            };

            let phonetic_password = if share.phonetic_enabled {
                Some(self.generate_phonetic_representation(&decrypted_password))
            } else {
                None
            };

            let otp_code = if let Some(encrypted_otp_secret) = share.otp_secret_encrypted {
                let otp_secret = self.encryption_service.decrypt(&encrypted_otp_secret)?;
                Some(self.generate_totp(&otp_secret)?)
            } else {
                None
            };

            // Update view count and last accessed
            let new_view_count = share.view_count + 1;
            let should_deactivate = share.one_time_use || 
                (share.max_views.is_some() && new_view_count >= share.max_views.unwrap());

            sqlx::query!(
                r#"
                UPDATE password_shares 
                SET view_count = $1, last_accessed = NOW(), is_active = $2
                WHERE id = $3
                "#,
                new_view_count,
                !should_deactivate,
                share.id
            )
            .execute(&self.db_pool)
            .await?;

            let remaining_views = share.max_views.map(|max| max - new_view_count);

            Ok(Some(PasswordShareAccessResponse {
                password_name: share.password_name.unwrap_or_else(|| "Shared Password".to_string()),
                password: decrypted_password,
                phonetic_password,
                username: share.username,
                url: share.url,
                notes: decrypted_notes,
                otp_code,
                expires_at: share.expires_at,
                remaining_views,
            }))
        } else {
            Ok(None)
        }
    }

    fn generate_secure_token(&self) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    pub async fn list_password_shares(&self, password_id: Option<Uuid>, created_by: Option<Uuid>) -> Result<Vec<PasswordShareResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let shares = sqlx::query!(
            r#"
            SELECT ps.*, p.name as password_name, u.name as created_by_name
            FROM password_shares ps
            JOIN passwords p ON ps.password_id = p.id
            JOIN users u ON ps.created_by = u.id
            WHERE ($1::uuid IS NULL OR ps.password_id = $1)
              AND ($2::uuid IS NULL OR ps.created_by = $2)
            ORDER BY ps.created_at DESC
            "#,
            password_id,
            created_by
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut results = Vec::new();
        for share in shares {
            let is_expired = share.expires_at <= Utc::now();
            let share_url = format!("/shared-password/{}", share.share_token); // Base URL will be added by frontend

            results.push(PasswordShareResponse {
                id: share.id,
                password_id: share.password_id,
                password_name: share.password_name.unwrap_or_else(|| "Unknown".to_string()),
                share_token: share.share_token,
                share_url,
                recipient_email: share.recipient_email,
                recipient_name: share.recipient_name,
                expires_at: share.expires_at,
                max_views: share.max_views,
                view_count: share.view_count,
                require_email_verification: share.require_email_verification,
                require_password: share.require_password,
                one_time_use: share.one_time_use,
                created_at: share.created_at,
                last_accessed: share.last_accessed,
                is_active: share.is_active,
                is_expired,
                created_by: share.created_by,
                created_by_name: share.created_by_name.unwrap_or_else(|| "Unknown".to_string()),
            });
        }

        Ok(results)
    }

    pub async fn list_passwords(&self, client_id: Option<Uuid>, folder_id: Option<Uuid>) -> Result<PasswordListResponse, Box<dyn std::error::Error + Send + Sync>> {
        let passwords = sqlx::query!(
            r#"
            SELECT p.id, p.client_id, c.name as client_name, p.name, p.description, p.username,
                   p.url, p.category, p.tags, p.favorite, p.otp_secret_encrypted,
                   p.phonetic_enabled, p.created_at, p.updated_at, p.last_accessed,
                   p.expires_at, p.strength_score, p.breach_detected, p.folder_id,
                   f.name as folder_name
            FROM passwords p
            LEFT JOIN clients c ON p.client_id = c.id
            LEFT JOIN password_folders f ON p.folder_id = f.id
            WHERE ($1::uuid IS NULL OR p.client_id = $1)
              AND ($2::uuid IS NULL OR p.folder_id = $2)
            ORDER BY p.name
            "#,
            client_id,
            folder_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let folders = sqlx::query!(
            r#"
            SELECT pf.id, pf.client_id, c.name as client_name, pf.name, pf.description,
                   pf.parent_id, parent.name as parent_name, pf.created_by, u.name as created_by_name,
                   pf.created_at, pf.updated_at, COUNT(p.id) as password_count
            FROM password_folders pf
            LEFT JOIN clients c ON pf.client_id = c.id
            LEFT JOIN password_folders parent ON pf.parent_id = parent.id
            LEFT JOIN users u ON pf.created_by = u.id
            LEFT JOIN passwords p ON pf.id = p.folder_id
            WHERE ($1::uuid IS NULL OR pf.client_id = $1)
            GROUP BY pf.id, pf.client_id, c.name, pf.name, pf.description, pf.parent_id,
                     parent.name, pf.created_by, u.name, pf.created_at, pf.updated_at
            ORDER BY pf.name
            "#,
            client_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let password_list: Vec<PasswordListItem> = passwords
            .into_iter()
            .map(|row| {
                let tags: Vec<String> = serde_json::from_str(&row.tags).unwrap_or_default();
                let expires_soon = if let Some(expires_at) = row.expires_at {
                    expires_at <= Utc::now() + Duration::days(30)
                } else {
                    false
                };

                PasswordListItem {
                    id: row.id,
                    client_id: row.client_id,
                    client_name: row.client_name,
                    name: row.name,
                    description: row.description,
                    username: row.username,
                    url: row.url,
                    category: row.category,
                    tags,
                    favorite: row.favorite,
                    has_otp: row.otp_secret_encrypted.is_some(),
                    phonetic_enabled: row.phonetic_enabled,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    last_accessed: row.last_accessed,
                    expires_at: row.expires_at,
                    expires_soon,
                    strength_score: row.strength_score,
                    strength_label: self.get_strength_label(row.strength_score),
                    breach_detected: row.breach_detected,
                    folder_id: row.folder_id,
                    folder_name: row.folder_name,
                }
            })
            .collect();

        let folder_list: Vec<PasswordFolderResponse> = folders
            .into_iter()
            .map(|row| PasswordFolderResponse {
                id: row.id,
                client_id: row.client_id,
                client_name: row.client_name,
                name: row.name,
                description: row.description,
                parent_id: row.parent_id,
                parent_name: row.parent_name,
                password_count: row.password_count.unwrap_or(0),
                created_by: row.created_by,
                created_by_name: row.created_by_name.unwrap_or_else(|| "Unknown".to_string()),
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(PasswordListResponse {
            passwords: password_list,
            folders: folder_list,
        })
    }
}