-- Create users table
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc(),
  username TEXT COLLATE "case_insensitive" UNIQUE NOT NULL,
  email TEXT COLLATE "case_insensitive" UNIQUE NOT NULL,
  email_verified BOOLEAN NOT NULL DEFAULT FALSE,
  password_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create trigger for users table
SELECT trigger_updated_at('users');

-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v1mc() NOT NULL,
  user_id UUID REFERENCES users(id),
  data JSONB,
  expiry_date TIMESTAMPTZ NOT NULL
);

-- Create email_verification_token table
CREATE TABLE IF NOT EXISTS email_verification_token (
  id TEXT PRIMARY KEY NOT NULL,
  active_expires TIMESTAMPTZ NOT NULL,
  user_id UUID NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE NO ACTION ON DELETE NO ACTION
);

-- Create password_reset_token table
CREATE TABLE IF NOT EXISTS password_reset_token (
  id TEXT PRIMARY KEY NOT NULL,
  active_expires TIMESTAMPTZ NOT NULL,
  user_id UUID NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE NO ACTION ON DELETE NO ACTION
);
