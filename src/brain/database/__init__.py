import os
import yaml
from datetime import datetime
from peewee import Proxy, CharField, DateTimeField, Model, PostgresqlDatabase, DecimalField, IntegerField, DateField
from brain.logger_util import logger


config_path = os.environ.get("BRAIN_CONFIG_PATH", "config.yaml")
db = Proxy()

if os.path.exists(config_path):
    with open(config_path, "r") as f:
        config = yaml.safe_load(f)
    db_settings = config["postgresql"]
    real_db = PostgresqlDatabase(**db_settings)
    db.initialize(real_db)
else:
    logger.warning(f"Config file not found at {config_path}. Database will remain uninitialized.")

class AlphaEntity(Model):
    alpha_id = CharField(max_length=50, primary_key=True)
    alpha_hashed = CharField(max_length=100)
    expression = CharField(max_length=255)
    dataset_id = CharField(max_length=50)
    create_time = DateTimeField(default=datetime.now)

    class Meta:
        database = db
        table_name = 'alpha_entity'

class YearStatsEntity(Model):
    year = CharField(max_length=20)
    pnl = DecimalField(max_digits=15, decimal_places=5)
    book_size = IntegerField()
    long_count = IntegerField()
    short_count = IntegerField()
    turnover = DecimalField(max_digits=15, decimal_places=5)
    sharpe = DecimalField(max_digits=15, decimal_places=5)
    returns = DecimalField(max_digits=15, decimal_places=5)
    drawdown = DecimalField(max_digits=15, decimal_places=5)
    margin = DecimalField(max_digits=15, decimal_places=5)
    fitness = DecimalField(max_digits=15, decimal_places=5)
    stage = CharField()
    alpha_id = CharField(max_length=50)

    class Meta:
        database = db
        table_name = 'year_stats_entity'
        # Define a composite index on
        indexes = (
            (('year', 'alpha_id'), False),
        )

class IsStatsEntity(Model):
    alpha_id = CharField(max_length=50, primary_key=True)
    pnl = DecimalField(max_digits=15, decimal_places=5)
    book_size = IntegerField()
    long_count = IntegerField()
    short_count = IntegerField()
    turnover = DecimalField(max_digits=15, decimal_places=5)
    returns = DecimalField(max_digits=15, decimal_places=5)
    drawdown = DecimalField(max_digits=15, decimal_places=5)
    margin = DecimalField(max_digits=15, decimal_places=5)
    sharpe = DecimalField(max_digits=15, decimal_places=5)
    fitness = DecimalField(max_digits=15, decimal_places=5)
    start_date = DateField()
    class Meta:
        database = db
        table_name = 'is_stats_entity'

class RiskNeutralizedEndity(Model):
    alpha_id = CharField(max_length=50, primary_key=True)
    pnl = DecimalField(max_digits=15, decimal_places=5)
    book_size = IntegerField()
    long_count = IntegerField()
    short_count = IntegerField()
    turnover = DecimalField(max_digits=15, decimal_places=5)
    returns = DecimalField(max_digits=15, decimal_places=5)
    drawdown = DecimalField(max_digits=15, decimal_places=5)
    margin = DecimalField(max_digits=15, decimal_places=5)
    fitness = DecimalField(max_digits=15, decimal_places=5)
    sharpe = DecimalField(max_digits=15, decimal_places=5)
    class Meta:
        database = db
        table_name = 'risk_neutralized_entity'

class InvestabilityConstrainedEntity(Model):
    alpha_id = CharField(max_length=50, primary_key=True)
    pnl = DecimalField(max_digits=15, decimal_places=5)
    book_size = IntegerField()
    long_count = IntegerField()
    short_count = IntegerField()
    turnover = DecimalField(max_digits=15, decimal_places=5)
    returns = DecimalField(max_digits=15, decimal_places=5)
    drawdown = DecimalField(max_digits=15, decimal_places=5)
    margin = DecimalField(max_digits=15, decimal_places=5)
    fitness = DecimalField(max_digits=15, decimal_places=5)
    sharpe = DecimalField(max_digits=15, decimal_places=5)
    class Meta:
        database = db
        table_name = 'investability_constrained_entity'

def init_tables():
    db.connect()
    db.create_tables([AlphaEntity, YearStatsEntity, IsStatsEntity, RiskNeutralizedEndity, InvestabilityConstrainedEntity])
