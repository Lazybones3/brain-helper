import hashlib
import json
from typing import List, Optional
from datetime import datetime, timezone
from brain.database import AlphaEntity
from brain.logger_util import logger

def hash_alpha(alpha_dict: dict) -> str:
    alpha_str = json.dumps(alpha_dict, sort_keys=True)
    return hashlib.sha256(alpha_str.encode("utf-8")).hexdigest()

class AlphaDao:
    @staticmethod
    def add_to_cache(alpha_dict: dict, alpha_id: str, dataset_id: str):
        alpha_hashed = hash_alpha(alpha_dict)
        existed = AlphaEntity.select().where(AlphaEntity.alpha_id==alpha_id).exists()
        if existed:
            logger.debug(f"alpha exists: {alpha_id}")
            return
        # Append new record
        new_row = AlphaEntity.create(
            alpha_hashed=alpha_hashed,
            alpha_id=alpha_id,
            expression=alpha_dict['regular'],
            dataset_id=dataset_id,
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
    
    @staticmethod
    def query_alphas_by_dataset(dataset_id: str) -> List[AlphaEntity]:
        query = AlphaEntity.select().where(AlphaEntity.dataset_id == dataset_id)
        return list(query)
    