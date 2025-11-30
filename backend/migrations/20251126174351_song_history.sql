-- Add migration script here
CREATE TABLE IF NOT EXISTS song_history (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    title TEXT NOT NULL,
    alternative_title TEXT NOT NULL,
    artist TEXT NOT NULL,
    artist_url TEXT NOT NULL,
    image_src TEXT NOT NULL,
    song_duration BIGINT NOT NULL,
    url TEXT NOT NULL,
    album TEXT,
    video_id TEXT NOT NULL,
    playlist_id TEXT NOT NULL,
    media_type TEXT NOT NULL,
    tags TEXT[] NOT NULL,
    played_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX user_songs_by_date ON song_history(user_id, played_at DESC);
