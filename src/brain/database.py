import yaml
import hashlib
import json
from typing import List, Optional

from peewee import CharField, DateTimeField, Model, PostgresqlDatabase
from datetime import datetime, timezone
from .logger_util import logger

with open("config.yaml", "r") as f:
    config = yaml.safe_load(f)

db_settings = config["postgresql"]
db = PostgresqlDatabase(**db_settings)

class AlphaEntity(Model):
    alpha_id = CharField(max_length=50, primary_key=True)
    alpha_hashed = CharField(max_length=100)
    expression = CharField(max_length=255)
    create_time = DateTimeField(default=datetime.now)

    class Meta:
        database = db
        table_name = 'alpha_entity'


def hash_alpha(alpha_dict: dict) -> str:
    alpha_str = json.dumps(alpha_dict, sort_keys=True)
    return hashlib.sha256(alpha_str.encode("utf-8")).hexdigest()

class AlphaDao:
    @staticmethod
    def add_to_cache(alpha_dict: dict, alpha_id: str):
        alpha_hashed = hash_alpha(alpha_dict)
        # Append new record
        new_row = AlphaEntity.create(
            alpha_hashed=alpha_hashed,
            alpha_id=alpha_id,
            expression=alpha_dict['regular'],
            create_time=datetime.now(timezone.utc),
        )
        logger.debug(f"add alpha to database: {new_row.alpha_id}")

    @staticmethod
    def check_if_alpha_already_simulated(alpha_dict: dict) -> Optional[str]:
        alpha_hashed = hash_alpha(alpha_dict)
        record = AlphaEntity.get_or_none(AlphaEntity.alpha_hashed == alpha_hashed)
        if record:
            return record.alpha_id
        return None
    
    @staticmethod
    def query_alphas_by_date(start: str, end: str) -> List[AlphaEntity]:
        fmt = "%Y-%m-%d %H:%M:%S"
        start_time = datetime.strptime(start, fmt)
        end_time = datetime.strptime(end, fmt)
        query = AlphaEntity.select().where(AlphaEntity.create_time.between(start_time, end_time))
        return list(query)

if __name__ == '__main__':
    db.connect()
    db.create_tables([AlphaEntity])
