"""Configuration management for strategy service."""

from pydantic_settings import BaseSettings
from functools import lru_cache


class Settings(BaseSettings):
    database_url: str
    nats_url: str = "nats://localhost:4222"
    redis_url: str = "redis://localhost:6379"
    jwt_secret: str
    token_expiry_minutes: int = 15

    class Config:
        env_file = ".env"
        case_sensitive = False


@lru_cache()
def get_settings() -> Settings:
    return Settings()
