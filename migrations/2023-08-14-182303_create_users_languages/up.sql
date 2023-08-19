CREATE TABLE
  users_languages (
    user_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    seconds BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, language_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (language_id) REFERENCES languages (id)
  );
