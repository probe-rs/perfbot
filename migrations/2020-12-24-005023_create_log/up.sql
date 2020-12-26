CREATE TABLE logs (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  probe VARCHAR NOT NULL,
  chip VARCHAR NOT NULL,
  os VARCHAR NOT NULL,
  protocol VARCHAR NOT NULL,
  protocol_speed INT NOT NULL,
  commit_hash VARCHAR NOT NULL,
  timestamp TIMESTAMP NOT NULL,
  kind TEXT CHECK(kind IN ('ram', 'flash')) NOT NULL,
  read_speed INTEGER NOT NULL,
  write_speed INTEGER NOT NULL
)