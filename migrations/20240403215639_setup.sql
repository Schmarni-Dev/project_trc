CREATE TABLE IF NOT EXISTS turtles (
        id INTEGER NOT NULL,
        name TEXT NOT NULL,
        inventory TEXT NOT NULL,
        position TEXT NOT NULL,
        orientation TEXT NOT NULL,
        fuel INTEGER NOT NULL,
        max_fuel INTEGER NOT NULL,
        world TEXT NOT NULL,
        PRIMARY KEY (world,id),
        FOREIGN KEY (world)
	REFERENCES worlds (name)
);

CREATE TABLE IF NOT EXISTS worlds (
	name TEXT NOT NULL UNIQUE PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS blocks (
        chunk_key INTEGER NOT NULL,
        id TEXT NOT NULL,
        world TEXT NOT NULL,
        world_pos TEXT NOT NULL,
        is_air BOOLEAN NOT NULL,
	PRIMARY KEY (world,world_pos),
	FOREIGN KEY (world)
	REFERENCES worlds (name)
);
