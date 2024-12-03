-- Create a schema named "amourithm"
CREATE SCHEMA IF NOT EXISTS amourithm;

-- Create the users table in the "amourithm" schema
CREATE TABLE IF NOT EXISTS amourithm.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),  -- Use UUID with default generation
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,           -- Email field, increased size for flexibility
    password VARCHAR(256) NOT NULL,               -- Password field with larger size to handle hashed passwords
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,  -- Timestamp of account creation
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,  -- Timestamp of last update
    CONSTRAINT users_email_check CHECK (email ~* '^[a-z0-9]+(?:[._%+-]*[a-z0-9]+)*@[a-z0-9.-]+\.[a-z]{2,}$')  -- Email format check
);

-- Create the usersdata table to store additional user information
CREATE TABLE IF NOT EXISTS amourithm.usersdata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES amourithm.users(id) ON DELETE CASCADE,  -- Foreign key to users table
    firstname VARCHAR(50) NULL,
    lastname VARCHAR(50) NULL,
    age INT CHECK (age >= 18),  -- Age must be 18 or older (common in dating apps)
    gender VARCHAR(20) NULL,   -- Gender field
    city VARCHAR(100) NULL, -- Location field (can be a city, state, or general area)
    bio TEXT NULL,             -- Bio or personal description
    profile_picture_url VARCHAR(256) NULL,  -- URL to profile picture (optional)
    -- interests TEXT[] NULL,     -- Array of interests (e.g., hobbies, preferences)
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,  -- Timestamp of record creation
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,  -- Timestamp of last update
    CONSTRAINT age_check CHECK (age >= 18)  -- Ensures users are of legal age
);

-- Optionally, create an index on the username for performance
CREATE INDEX IF NOT EXISTS idx_username ON amourithm.users(username);

-- Optionally, create an index on the user_id for faster lookup in the usersdata table
CREATE INDEX IF NOT EXISTS idx_user_id ON amourithm.usersdata(user_id);
