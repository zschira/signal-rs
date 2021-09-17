CREATE TABLE messages (
  timestamp BIGINT NOT NULL,
  number TEXT,
  from_me TINYINT NOT NULL,
  is_read TINYINT NOT NULL,
  attachments TEXT,
  body TEXT NOT NULL,
  groupid TEXT,
  quote_timestamp BIGINT,
  quote_author TEXT,
  mentions BLOB,
  mentions_start BLOB,
  reaction_emojis TEXT,
  reaction_authors TEXT,
  PRIMARY KEY (timestamp, from_me, number, groupid)
)
