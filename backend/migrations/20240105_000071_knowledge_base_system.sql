-- Comprehensive Knowledge Base System
-- Rich text editor, templates, Global KB, Client KB, versioning, approval workflow

-- KB Categories and organization
CREATE TABLE kb_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE, -- NULL for global categories
    name VARCHAR(255) NOT NULL,
    description TEXT,
    slug VARCHAR(255) NOT NULL,
    icon VARCHAR(50),
    color VARCHAR(7), -- Hex color
    parent_category_id UUID REFERENCES kb_categories(id) ON DELETE SET NULL,
    sort_order INTEGER DEFAULT 0,
    is_visible BOOLEAN DEFAULT true,
    is_system_category BOOLEAN DEFAULT false,
    article_count INTEGER DEFAULT 0,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(client_id, slug)
);

-- Insert default system categories
INSERT INTO kb_categories (name, description, slug, icon, color, is_system_category, created_by) VALUES
('General', 'General knowledge base articles', 'general', 'info', '#6c757d', true, (SELECT id FROM users LIMIT 1)),
('Hardware', 'Hardware documentation and guides', 'hardware', 'server', '#007bff', true, (SELECT id FROM users LIMIT 1)),
('Software', 'Software installation and configuration', 'software', 'download', '#28a745', true, (SELECT id FROM users LIMIT 1)),
('Network', 'Network configuration and troubleshooting', 'network', 'network-wired', '#17a2b8', true, (SELECT id FROM users LIMIT 1)),
('Security', 'Security procedures and policies', 'security', 'shield', '#dc3545', true, (SELECT id FROM users LIMIT 1)),
('Backup', 'Backup and disaster recovery procedures', 'backup', 'database', '#ffc107', true, (SELECT id FROM users LIMIT 1)),
('Troubleshooting', 'Common issues and solutions', 'troubleshooting', 'tools', '#fd7e14', true, (SELECT id FROM users LIMIT 1)),
('Procedures', 'Standard operating procedures', 'procedures', 'list-check', '#6f42c1', true, (SELECT id FROM users LIMIT 1));

-- KB Articles with versioning
CREATE TABLE kb_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE, -- NULL for global articles
    category_id UUID NOT NULL REFERENCES kb_categories(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL,
    content TEXT NOT NULL, -- Rich text content (HTML)
    content_plain TEXT, -- Plain text for search
    excerpt TEXT, -- Brief summary
    featured_image_url TEXT,
    status VARCHAR(20) DEFAULT 'draft', -- draft, pending, published, archived
    visibility VARCHAR(20) DEFAULT 'internal', -- internal, client, public
    article_type VARCHAR(20) DEFAULT 'article', -- article, procedure, faq, policy
    priority INTEGER DEFAULT 0, -- For featured articles
    view_count INTEGER DEFAULT 0,
    helpful_count INTEGER DEFAULT 0,
    not_helpful_count INTEGER DEFAULT 0,
    tags TEXT[] DEFAULT '{}',
    related_articles UUID[] DEFAULT '{}',
    approval_status VARCHAR(20) DEFAULT 'pending', -- pending, approved, rejected
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    approval_notes TEXT,
    published_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    last_reviewed TIMESTAMPTZ,
    review_required_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}', -- Custom metadata
    search_vector tsvector, -- Full-text search
    version INTEGER DEFAULT 1,
    is_latest_version BOOLEAN DEFAULT true,
    parent_article_id UUID REFERENCES kb_articles(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    UNIQUE(client_id, slug, version)
);

-- KB Templates for common article types
CREATE TABLE kb_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE, -- NULL for global templates
    name VARCHAR(255) NOT NULL,
    description TEXT,
    template_type VARCHAR(50) NOT NULL, -- procedure, faq, policy, incident_response, etc.
    content_template TEXT NOT NULL, -- HTML template with placeholders
    fields JSONB DEFAULT '{}', -- Template field definitions
    is_system_template BOOLEAN DEFAULT false,
    usage_count INTEGER DEFAULT 0,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- Insert default templates
INSERT INTO kb_templates (name, description, template_type, content_template, fields, is_system_template, created_by) VALUES
('Standard Operating Procedure', 'Template for creating SOPs', 'procedure', 
'<h1>{{title}}</h1>
<h2>Purpose</h2>
<p>{{purpose}}</p>
<h2>Scope</h2>
<p>{{scope}}</p>
<h2>Prerequisites</h2>
<ul>{{prerequisites}}</ul>
<h2>Procedure</h2>
<ol>{{procedure_steps}}</ol>
<h2>Verification</h2>
<p>{{verification}}</p>
<h2>Troubleshooting</h2>
<p>{{troubleshooting}}</p>', 
'{"title": "text", "purpose": "textarea", "scope": "textarea", "prerequisites": "list", "procedure_steps": "list", "verification": "textarea", "troubleshooting": "textarea"}',
true, (SELECT id FROM users LIMIT 1)),

('FAQ Article', 'Template for frequently asked questions', 'faq',
'<h1>{{title}}</h1>
<h2>Question</h2>
<p><strong>{{question}}</strong></p>
<h2>Answer</h2>
<p>{{answer}}</p>
<h2>Additional Information</h2>
<p>{{additional_info}}</p>
<h2>Related Articles</h2>
<ul>{{related_links}}</ul>',
'{"title": "text", "question": "text", "answer": "rich_text", "additional_info": "textarea", "related_links": "list"}',
true, (SELECT id FROM users LIMIT 1)),

('Troubleshooting Guide', 'Template for troubleshooting procedures', 'troubleshooting',
'<h1>{{title}}</h1>
<h2>Problem Description</h2>
<p>{{problem_description}}</p>
<h2>Symptoms</h2>
<ul>{{symptoms}}</ul>
<h2>Possible Causes</h2>
<ul>{{causes}}</ul>
<h2>Solution Steps</h2>
<ol>{{solution_steps}}</ol>
<h2>Prevention</h2>
<p>{{prevention}}</p>
<h2>Escalation</h2>
<p>{{escalation}}</p>',
'{"title": "text", "problem_description": "textarea", "symptoms": "list", "causes": "list", "solution_steps": "list", "prevention": "textarea", "escalation": "textarea"}',
true, (SELECT id FROM users LIMIT 1));

-- Article revisions/versions history
CREATE TABLE kb_article_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    change_summary TEXT,
    changed_by UUID NOT NULL REFERENCES users(id),
    changed_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(article_id, version)
);

-- Article access and permissions
CREATE TABLE kb_article_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    permission_type VARCHAR(20) NOT NULL, -- user, role, client, public
    permission_target VARCHAR(255), -- user_id, role_name, client_id, 'public'
    access_level VARCHAR(20) DEFAULT 'read', -- read, write, admin
    granted_by UUID NOT NULL REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    UNIQUE(article_id, permission_type, permission_target)
);

-- Article feedback and ratings
CREATE TABLE kb_article_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    rating INTEGER CHECK(rating BETWEEN 1 AND 5),
    is_helpful BOOLEAN,
    feedback_text TEXT,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Article attachments (links to file_attachments)
CREATE TABLE kb_article_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    file_attachment_id UUID NOT NULL REFERENCES file_attachments(id) ON DELETE CASCADE,
    display_name VARCHAR(255),
    description TEXT,
    display_order INTEGER DEFAULT 0,
    is_inline BOOLEAN DEFAULT false, -- Embedded in content vs attachment
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(article_id, file_attachment_id)
);

-- Article workflow and approval
CREATE TABLE kb_approval_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE, -- NULL for global workflow
    name VARCHAR(255) NOT NULL,
    description TEXT,
    workflow_steps JSONB NOT NULL, -- Array of approval steps
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default approval workflow
INSERT INTO kb_approval_workflows (name, description, workflow_steps, is_default, is_active, created_by) VALUES
('Standard Review', 'Standard article review and approval process',
'[
    {"step": 1, "name": "Peer Review", "approver_role": "editor", "required": true},
    {"step": 2, "name": "Technical Review", "approver_role": "technical_lead", "required": false},
    {"step": 3, "name": "Final Approval", "approver_role": "admin", "required": true}
]',
true, true, (SELECT id FROM users LIMIT 1));

CREATE TABLE kb_approval_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES kb_approval_workflows(id),
    current_step INTEGER DEFAULT 1,
    status VARCHAR(20) DEFAULT 'pending', -- pending, approved, rejected
    requested_by UUID NOT NULL REFERENCES users(id),
    requested_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    notes TEXT
);

CREATE TABLE kb_approval_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    approval_request_id UUID NOT NULL REFERENCES kb_approval_requests(id) ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    approver_id UUID REFERENCES users(id),
    status VARCHAR(20) DEFAULT 'pending', -- pending, approved, rejected, skipped
    comments TEXT,
    completed_at TIMESTAMPTZ,
    UNIQUE(approval_request_id, step_number)
);

-- Article analytics and views
CREATE TABLE kb_article_views (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES kb_articles(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ip_address INET,
    user_agent TEXT,
    referrer TEXT,
    view_duration_seconds INTEGER,
    viewed_at TIMESTAMPTZ DEFAULT NOW()
);

-- Search queries for analytics
CREATE TABLE kb_search_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    query_text TEXT NOT NULL,
    results_count INTEGER DEFAULT 0,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    clicked_article_id UUID REFERENCES kb_articles(id) ON DELETE SET NULL,
    ip_address INET,
    searched_at TIMESTAMPTZ DEFAULT NOW()
);

-- Comprehensive indexing for performance
CREATE INDEX idx_kb_categories_client ON kb_categories(client_id);
CREATE INDEX idx_kb_categories_parent ON kb_categories(parent_category_id);
CREATE INDEX idx_kb_categories_slug ON kb_categories(client_id, slug);

CREATE INDEX idx_kb_articles_client_category ON kb_articles(client_id, category_id);
CREATE INDEX idx_kb_articles_status ON kb_articles(status, visibility);
CREATE INDEX idx_kb_articles_slug ON kb_articles(client_id, slug);
CREATE INDEX idx_kb_articles_published ON kb_articles(published_at DESC) WHERE status = 'published';
CREATE INDEX idx_kb_articles_tags ON kb_articles USING gin(tags);
CREATE INDEX idx_kb_articles_search ON kb_articles USING gin(search_vector);
CREATE INDEX idx_kb_articles_latest ON kb_articles(parent_article_id, version DESC) WHERE is_latest_version = true;

CREATE INDEX idx_kb_templates_client_type ON kb_templates(client_id, template_type);

CREATE INDEX idx_kb_article_revisions_article ON kb_article_revisions(article_id, version);
CREATE INDEX idx_kb_article_permissions_article ON kb_article_permissions(article_id);
CREATE INDEX idx_kb_article_feedback_article ON kb_article_feedback(article_id);
CREATE INDEX idx_kb_article_views_article ON kb_article_views(article_id, viewed_at);
CREATE INDEX idx_kb_search_queries_client ON kb_search_queries(client_id, searched_at);

-- Full-text search trigger
CREATE OR REPLACE FUNCTION update_kb_article_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.content_plain, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(NEW.excerpt, '')), 'C') ||
        setweight(to_tsvector('english', array_to_string(NEW.tags, ' ')), 'D');
    
    -- Update plain text content from HTML
    NEW.content_plain := regexp_replace(NEW.content, '<[^>]*>', '', 'g');
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_kb_article_search_vector_trigger
    BEFORE INSERT OR UPDATE ON kb_articles
    FOR EACH ROW EXECUTE FUNCTION update_kb_article_search_vector();

-- Article view count increment
CREATE OR REPLACE FUNCTION increment_kb_article_views(article_uuid UUID)
RETURNS void AS $$
BEGIN
    UPDATE kb_articles 
    SET view_count = view_count + 1 
    WHERE id = article_uuid;
END;
$$ LANGUAGE plpgsql;

-- Category article count maintenance
CREATE OR REPLACE FUNCTION update_kb_category_article_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE kb_categories 
        SET article_count = article_count + 1 
        WHERE id = NEW.category_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE kb_categories 
        SET article_count = article_count - 1 
        WHERE id = OLD.category_id;
    ELSIF TG_OP = 'UPDATE' AND NEW.category_id != OLD.category_id THEN
        UPDATE kb_categories 
        SET article_count = article_count - 1 
        WHERE id = OLD.category_id;
        UPDATE kb_categories 
        SET article_count = article_count + 1 
        WHERE id = NEW.category_id;
    END IF;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_kb_category_article_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON kb_articles
    FOR EACH ROW EXECUTE FUNCTION update_kb_category_article_count();

-- Update triggers for timestamps
CREATE TRIGGER update_kb_categories_updated_at 
    BEFORE UPDATE ON kb_categories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

CREATE TRIGGER update_kb_articles_updated_at 
    BEFORE UPDATE ON kb_articles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

CREATE TRIGGER update_kb_templates_updated_at 
    BEFORE UPDATE ON kb_templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_generic();

-- Cleanup old data
CREATE OR REPLACE FUNCTION cleanup_kb_analytics()
RETURNS void AS $$
BEGIN
    DELETE FROM kb_article_views 
    WHERE viewed_at < NOW() - INTERVAL '2 years';
    
    DELETE FROM kb_search_queries 
    WHERE searched_at < NOW() - INTERVAL '1 year';
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE kb_articles IS 'Knowledge base articles with versioning and approval workflow';
COMMENT ON TABLE kb_templates IS 'Article templates for consistent documentation';
COMMENT ON TABLE kb_approval_workflows IS 'Configurable approval processes for article publishing';
COMMENT ON COLUMN kb_articles.search_vector IS 'Full-text search vector for article content';
COMMENT ON COLUMN kb_articles.content IS 'Rich HTML content of the article';
COMMENT ON COLUMN kb_articles.content_plain IS 'Plain text version for search and previews';