-- Initial database setup for Resolve Demo
-- This runs before migrations to ensure proper structure

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create initial admin user table if not exists (will be enhanced by migrations)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    role VARCHAR(50) DEFAULT 'user',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create initial clients table if not exists
CREATE TABLE IF NOT EXISTS clients (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    website VARCHAR(255),
    address TEXT,
    city VARCHAR(100),
    state VARCHAR(100),
    zip VARCHAR(20),
    country VARCHAR(100) DEFAULT 'USA',
    client_type VARCHAR(50) DEFAULT 'business',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert demo admin user (password: demo123)
INSERT INTO users (email, password_hash, first_name, last_name, role, is_active)
VALUES ('admin@resolve.demo', '$2b$12$LQv3c1yqBwEHxv68UVgAiO1.Q0IKEWKhLzxg2.fGQK8BmL.3K9FX6', 'Admin', 'User', 'admin', true)
ON CONFLICT (email) DO NOTHING;

SELECT 'Database initialized successfully' as status;