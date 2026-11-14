/*"
        CREATE TABLE IF NOT EXISTS bot_constantes_variables (
            guild_id INTEGER NOT NULL,
            cle TEXT NOT NULL,
            valeur TEXT NOT NULL,
            is_constant BOOLEAN DEFAULT 0,
            PRIMARY KEY (guild_id, cle)
        )"