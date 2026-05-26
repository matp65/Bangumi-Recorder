-- Normalize all existing tokens from u64::MAX (~0) to ALL_COMBINED (255)
-- so they're compatible with JavaScript frontend bitwise operations
UPDATE api_tokens SET permissions = 255 WHERE permissions = ~0;
