-- [AR] - SQL Migration
UPDATE minecraft_users SET account_type = 'offline' WHERE account_type == 'pirate';
