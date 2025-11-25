-- Client 4-Character Identifiers and Logo System
-- Adds professional branding features to client management

DO $$
BEGIN
    -- Add client branding columns if they don't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'clients' AND column_name = 'client_code') THEN
        ALTER TABLE clients ADD COLUMN client_code CHAR(4) UNIQUE;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'clients' AND column_name = 'logo_filename') THEN
        ALTER TABLE clients ADD COLUMN logo_filename VARCHAR(255);
        ALTER TABLE clients ADD COLUMN logo_file_path TEXT;
        ALTER TABLE clients ADD COLUMN logo_mime_type VARCHAR(100);
        ALTER TABLE clients ADD COLUMN logo_file_size INTEGER;
        ALTER TABLE clients ADD COLUMN primary_color VARCHAR(7);
        ALTER TABLE clients ADD COLUMN secondary_color VARCHAR(7);
        ALTER TABLE clients ADD COLUMN brand_guidelines TEXT;
        ALTER TABLE clients ADD COLUMN logo_uploaded_at TIMESTAMPTZ;
        ALTER TABLE clients ADD COLUMN logo_uploaded_by UUID;
    END IF;
END $$;

-- Function to generate 4-character client codes
CREATE OR REPLACE FUNCTION generate_client_code(client_name TEXT)
RETURNS CHAR(4) AS $$
DECLARE
    proposed_code CHAR(4);
    name_words TEXT[];
    word TEXT;
    code_chars TEXT := '';
    counter INTEGER := 0;
    suffix INTEGER := 1;
BEGIN
    -- Clean and split the client name
    name_words := string_to_array(
        regexp_replace(upper(regexp_replace(client_name, '[^a-zA-Z\s]', '', 'g')), '\s+', ' ', 'g'), 
        ' '
    );
    
    -- Use first letter of each word (up to 4 words)
    FOREACH word IN ARRAY name_words LOOP
        IF length(word) > 0 AND length(code_chars) < 4 THEN
            code_chars := code_chars || left(word, 1);
        END IF;
    END LOOP;
    
    -- If less than 4 chars, use more from first word
    IF length(code_chars) < 4 AND array_length(name_words, 1) > 0 THEN
        word := name_words[1];
        code_chars := left(word, 4);
    END IF;
    
    -- Pad to 4 characters if needed
    WHILE length(code_chars) < 4 LOOP
        code_chars := code_chars || left(coalesce(name_words[1], 'X'), 1);
    END LOOP;
    
    proposed_code := left(code_chars, 4);
    
    -- Handle conflicts
    WHILE EXISTS(SELECT 1 FROM clients WHERE client_code = proposed_code) LOOP
        IF suffix <= 9 THEN
            proposed_code := left(code_chars, 3) || suffix::TEXT;
        ELSE
            proposed_code := left(md5(client_name || current_timestamp::text), 4);
            EXIT;
        END IF;
        suffix := suffix + 1;
        
        IF suffix > 99 THEN
            proposed_code := left(md5(client_name || current_timestamp::text), 4);
            EXIT;
        END IF;
    END LOOP;
    
    RETURN upper(proposed_code);
END;
$$ LANGUAGE plpgsql;

-- Generate codes for existing clients without codes
DO $$
DECLARE
    client_record RECORD;
    new_code CHAR(4);
BEGIN
    FOR client_record IN 
        SELECT id, name FROM clients WHERE client_code IS NULL
    LOOP
        new_code := generate_client_code(client_record.name);
        UPDATE clients SET client_code = new_code WHERE id = client_record.id;
    END LOOP;
END $$;

-- Add indexes
CREATE INDEX IF NOT EXISTS idx_clients_client_code ON clients(client_code) WHERE client_code IS NOT NULL;