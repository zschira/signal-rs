CREATE TABLE messages (
  timestamp BIGINT NOT NULL,
  number TEXT,
  from_me TINYINT NOT NULL,
  attachments TEXT,
  body TEXT NOT NULL,
  groupid TEXT,
  quote_timestamp BIGINT,
  quote_author TEXT,
  mentions BLOB,
  mentions_start BLOB,
  PRIMARY KEY (timestamp, from_me, number, groupid)
)
