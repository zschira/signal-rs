CREATE TABLE messages (
  timestamp BIGINT NOT NULL,
  number TEXT NOT NULL,
  attachments TEXT,
  body TEXT NOT NULL,
  groupid TEXT,
  quote_timestamp BIGINT,
  quote_uuid BLOB,
  mentions BLOB,
  PRIMARY KEY (timestamp, number, groupid)
)
