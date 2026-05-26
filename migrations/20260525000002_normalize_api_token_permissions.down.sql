-- Revert: set permissions back to u64::MAX (~0) for tokens that currently have ALL_COMBINED (255)
UPDATE api_tokens SET permissions = ~0 WHERE permissions = 255;
