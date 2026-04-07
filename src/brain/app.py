from datetime import datetime, timedelta
import random
from typing import Literal, Optional
from itertools import product
import pandas as pd
from tqdm import tqdm

from brain.logger_util import logger
from brain.api import BrainApi
from brain.database.alpha_dao import AlphaDao
from brain.database.year_stats_dao import YearStatsDao
from brain.database.is_stats_dao import IsStatsDao
from brain.database.risk_neutralized_dao import RiskNeutralizedDao
from brain.database.investability_constrained_dao import InvestabilityConstrainedDao


class BrainApp:
    def __init__(
        self,
        dataset_id: str,
        region: str = "USA",
        universe: str = "TOP3000",
        delay: Literal[0, 1] = 1,
        decay: int = 0,
        neutralization: str = "SUBINDUSTRY",
        truncation: float = 0.1,
        pasteurization: Literal["ON", "OFF"] = "ON",
        test_period: str = "P0Y0M0D",
        unit_handling: Literal["VERIFY"] = "VERIFY",
        nan_handling: Literal["ON", "OFF"] = "OFF",
        visualization: bool = False,
        alpha_type: Literal["REGULAR", "SUPER"] = "REGULAR",
        limit_of_concurrent_simulations: int = 3,
        limit_of_multi_simulations: int = 3,
        api: Optional[BrainApi] = None,
    ) -> None:
        self.dataset_id: str = dataset_id
        self.region: str = region
        self.universe: str = universe
        self.delay: Literal[0, 1] = delay
        self.decay: int = decay
        self.neutralization: str = neutralization
        self.truncation: float = truncation
        self.pasteurization: Literal["ON", "OFF"] = pasteurization
        self.test_period: str = test_period
        self.unit_handling: Literal["VERIFY"] = unit_handling
        self.nan_handling: Literal["ON", "OFF"] = nan_handling
        self.visualization: bool = visualization
        self.alpha_type: Literal["REGULAR", "SUPER"] = alpha_type
        self.limit_of_concurrent_simulations: int = limit_of_concurrent_simulations
        self.limit_of_multi_simulations: int = limit_of_multi_simulations
        if api:
            self.api: BrainApi = api
        else:
            self.api: BrainApi = BrainApi()
        self.expiry_time = None
        self.set_expiry_time()

        basic_ops = ["reverse", "inverse", "rank", "zscore", "quantile", "normalize"]
        ts_ops = [
            "ts_rank",
            "ts_zscore",
            "ts_delta",
            "ts_sum",
            "ts_delay",
            "ts_std_dev",
            "ts_mean",
            "ts_arg_min",
            "ts_arg_max",
            "ts_scale",
            "ts_quantile",
        ]
        self.ops_set = basic_ops + ts_ops

    def set_expiry_time(self):
        token_expiry = self.api.check_session_timeout()
        self.expiry_time = datetime.now() + timedelta(seconds=token_expiry)

    def _is_close_to_expiry(self):
        if self.expiry_time is None:
            return True
        return datetime.now() >= (self.expiry_time - timedelta(seconds=1000))

    def process_alpha(
        self,
        expression_list: list[str],
    ) -> list:
        alpha_list = [
            self.api.generate_alpha(
                x,
                alpha_type=self.alpha_type,
                region=self.region,
                universe=self.universe,
                delay=self.delay,
                decay=self.decay,
                neutralization=self.neutralization,
                truncation=self.truncation,
                pasteurization=self.pasteurization,
                test_period=self.test_period,
                unit_handling=self.unit_handling,
                nan_handling=self.nan_handling,
                visualization=self.visualization,
            )
            for x in expression_list
        ]

        new_alpha_list = []
        reused_alpha_ids = []

        for alpha_dict in alpha_list:
            existing_alpha_id = AlphaDao.check_if_alpha_already_simulated(alpha_dict)
            if existing_alpha_id is not None:
                reused_alpha_ids.append(
                    {"alpha_id": existing_alpha_id, "simulate_data": alpha_dict}
                )
            else:
                new_alpha_list.append(alpha_dict)

        logger.info(
            f"Found {len(reused_alpha_ids)} duplicates, {len(new_alpha_list)} new alphas to simulate."
        )
        return new_alpha_list

    def simulate_alpha(self, alpha_list: list, batch_size: int):
        sim_alphas = [
            alpha_list[i : i + batch_size]
            for i in range(0, len(alpha_list), batch_size)
        ]
        total_alpha = len(sim_alphas)
        with tqdm(total=total_alpha * batch_size, desc="Simulation Progress") as pbar:
            for _ in range(total_alpha):
                if self._is_close_to_expiry():
                    self.api.check_session_and_relogin()
                    self.set_expiry_time()
                alphas = random.choice(sim_alphas)
                result_list = self.api.simulate_alpha_list_multi(
                    alphas,
                    limit_of_concurrent_simulations=self.limit_of_concurrent_simulations,
                    limit_of_multi_simulations=self.limit_of_multi_simulations,
                )
                for result in result_list:
                    if "alpha_id" not in result or result["alpha_id"] is None:
                        logger.error(f"alpha_id is null, check alphas: {alphas}")
                        continue
                    alpha_id = result["alpha_id"]
                    regular = result["simulate_data"]["regular"]
                    filtered_alphas = list(
                        filter(lambda item: item["regular"] == regular, alphas)
                    )
                    if len(filtered_alphas) > 0:
                        self.save_simulation_result(result)
                        AlphaDao.add_to_cache(
                            filtered_alphas[0], alpha_id, self.dataset_id
                        )
                        logger.info(
                            f"simulate completed: {alpha_id}, regular: {regular}"
                        )
                    else:
                        logger.warning(f"Cannot found {regular} in: {alphas}")
                    pbar.update(1)

    def simulation(self, expression_list: list[str], batch_size: int = 10):
        alpha_list = self.process_alpha(expression_list)
        if len(alpha_list) == 0:
            return
        self.simulate_alpha(alpha_list, batch_size)

    def save_simulation_result(self, result):
        alpha_id = result["alpha_id"]
        is_stats_df = result["is_stats"]
        risk_series = is_stats_df["riskNeutralized"]
        investability_series = is_stats_df["investabilityConstrained"]
        IsStatsDao.insert(
            {
                "alpha_id": alpha_id,
                "pnl": is_stats_df.at[0, "pnl"].item(),
                "book_size": is_stats_df.at[0, "bookSize"].item(),
                "long_count": is_stats_df.at[0, "longCount"].item(),
                "short_count": is_stats_df.at[0, "shortCount"].item(),
                "turnover": is_stats_df.at[0, "turnover"].item(),
                "returns": is_stats_df.at[0, "returns"].item(),
                "drawdown": is_stats_df.at[0, "drawdown"].item(),
                "margin": is_stats_df.at[0, "margin"].item(),
                "sharpe": is_stats_df.at[0, "sharpe"].item(),
                "fitness": is_stats_df.at[0, "fitness"].item(),
                "start_date": is_stats_df.at[0, "startDate"],
            }
        )
        if len(risk_series) > 0:
            RiskNeutralizedDao.insert(
                {
                    "alpha_id": alpha_id,
                    "pnl": risk_series[0]["pnl"],
                    "book_size": risk_series[0]["bookSize"],
                    "long_count": risk_series[0]["longCount"],
                    "short_count": risk_series[0]["shortCount"],
                    "turnover": risk_series[0]["turnover"],
                    "returns": risk_series[0]["returns"],
                    "drawdown": risk_series[0]["drawdown"],
                    "margin": risk_series[0]["margin"],
                    "sharpe": risk_series[0]["sharpe"],
                    "fitness": risk_series[0]["fitness"],
                }
            )
        if len(investability_series) > 0:
            InvestabilityConstrainedDao.insert(
                {
                    "alpha_id": alpha_id,
                    "pnl": investability_series[0]["pnl"],
                    "book_size": investability_series[0]["bookSize"],
                    "long_count": investability_series[0]["longCount"],
                    "short_count": investability_series[0]["shortCount"],
                    "turnover": investability_series[0]["turnover"],
                    "returns": investability_series[0]["returns"],
                    "drawdown": investability_series[0]["drawdown"],
                    "margin": investability_series[0]["margin"],
                    "sharpe": investability_series[0]["sharpe"],
                    "fitness": investability_series[0]["fitness"],
                }
            )

    def save_alpha_yearly_stats(self):
        alphas = AlphaDao.query_alphas_by_dataset(self.dataset_id)
        with tqdm(total=len(alphas), desc="Saving Stats Progress") as pbar:
            for alpha in alphas:
                df = self.api.get_alpha_yearly_stats(alpha_id=str(alpha.alpha_id))
                for _, row in df.iterrows():
                    YearStatsDao.insert(
                        {
                            "year": row["year"],
                            "pnl": row["pnl"],
                            "book_size": row["bookSize"],
                            "long_count": row["longCount"],
                            "short_count": row["shortCount"],
                            "turnover": row["turnover"],
                            "sharpe": row["sharpe"],
                            "returns": row["returns"],
                            "drawdown": row["drawdown"],
                            "margin": row["margin"],
                            "fitness": row["fitness"],
                            "stage": row["stage"],
                            "alpha_id": row["alpha_id"],
                        }
                    )
                pbar.update(1)

    def get_dataset_id(
        self,
        name: str,
        theme: Optional[bool] = None,
    ) -> Optional[str]:
        datasets_df = self.api.get_datasets(
            region=self.region, delay=self.delay, universe=self.universe, theme=theme
        )
        selected_datasets_df = datasets_df[
            datasets_df["name"].str.contains(name, case=False)
        ]
        if len(selected_datasets_df) == 0:
            return None
        return str(selected_datasets_df.id.values.tolist()[0])

    def get_datafields_by_dataset_id(
        self, type: Optional[Literal["MATRIX", "VECTOR"]] = None, search: str = ""
    ) -> pd.DataFrame:
        datafields_df = self.api.get_datafields(
            region=self.region, delay=self.delay, universe=self.universe, search=search
        )
        if type:
            filtered_datafields_df = datafields_df[
                (datafields_df["dataset_id"] == self.dataset_id)
                & (datafields_df["type"] == type)
            ].reset_index(drop=True)
        else:
            filtered_datafields_df = datafields_df[
                (datafields_df["dataset_id"] == self.dataset_id)
            ].reset_index(drop=True)
        return filtered_datafields_df

    def get_datasets_field_list(
        self, type: Literal["MATRIX", "VECTOR"] = "MATRIX"
    ) -> list:
        # dataset_id = self.get_dataset_id(dataset_name, theme=theme)
        # logger.info(f"dataset_id: {dataset_id}")
        # if not dataset_id:
        #     return []
        datafileds_df = self.get_datafields_by_dataset_id(type=type)
        field_list = datafileds_df.id.values.tolist()
        logger.info(f"total data fields: {len(field_list)}")
        return field_list

    def get_operators_desc(
        self, scope: Literal["COMBO", "REGULAR", "SELECTION"] = "REGULAR"
    ) -> pd.DataFrame:
        operators = self.api.get_operators()
        operators = operators[operators["scope"] == scope]
        operators_info = operators[["definition", "category", "description"]]
        return operators_info

    def get_datafields_desc(
        self, type: Literal["MATRIX", "VECTOR"] = "MATRIX"
    ) -> Optional[pd.DataFrame]:
        # dataset_id = self.get_dataset_id(dataset_name, theme=theme)
        # if not dataset_id:
        #     logger.warning(f"dataset_id not found: {dataset_name}")
        #     return None
        datafields_df = self.get_datafields_by_dataset_id(type=type)
        datafields_info = datafields_df[
            ["id", "description", "type", "subcategory_name"]
        ]
        return datafields_info

    def get_vec_fields(self, fields):
        # 请在此处添加获得权限的Vector操作符
        vec_ops = ["vec_avg", "vec_sum"]
        vec_fields = []

        for field in fields:
            for vec_op in vec_ops:
                if vec_op == "vec_choose":
                    vec_fields.append("%s(%s, nth=-1)" % (vec_op, field))
                    vec_fields.append("%s(%s, nth=0)" % (vec_op, field))
                else:
                    vec_fields.append("%s(%s)" % (vec_op, field))

        return vec_fields

    def process_datafields(self, df):
        datafields = []
        datafields += df[df["type"] == "MATRIX"]["id"].tolist()
        datafields += self.get_vec_fields(df[df["type"] == "VECTOR"]["id"].tolist())
        return [
            "winsorize(ts_backfill(%s, 120), std=4)" % field for field in datafields
        ]

    def ts_comp_factory(self, op, field, factor, paras):
        output = []
        # l1, l2 = [3, 5, 10, 20, 60, 120, 240], paras
        l1, l2 = [5, 22, 66, 240], paras
        comb = list(product(l1, l2))
        for day, para in comb:
            if type(para) == float:
                alpha = "%s(%s, %d, %s=%.1f)" % (op, field, day, factor, para)
                output.append(alpha)
            elif type(para) == int:
                alpha = "%s(%s, %d, %s=%d)" % (op, field, day, factor, para)
                output.append(alpha)
        return output

    def vector_factory(self, op, field):
        output = []
        vectors = ["cap"]

        for vector in vectors:
            alpha = "%s(%s, %s)" % (op, field, vector)
            output.append(alpha)
        return output

    def ts_factory(self, op, field):
        output = []
        # days = [3, 5, 10, 20, 60, 120, 240]
        days = [5, 22, 66, 120, 240]

        for day in days:

            alpha = "%s(%s, %d)" % (op, field, day)
            output.append(alpha)

        return output

    def first_order_factory(self, fields):
        alpha_set = []
        # for field in fields:
        for field in fields:
            # reverse op does the work
            alpha_set.append(field)
            # alpha_set.append("-%s"%field)
            for op in self.ops_set:

                if op == "ts_percentage":

                    alpha_set += self.ts_comp_factory(op, field, "percentage", [0.5])

                elif op == "ts_decay_exp_window":

                    alpha_set += self.ts_comp_factory(op, field, "factor", [0.5])

                elif op == "ts_moment":

                    alpha_set += self.ts_comp_factory(op, field, "k", [2, 3, 4])

                elif op == "ts_entropy":

                    alpha_set += self.ts_comp_factory(op, field, "buckets", [10])

                elif op.startswith("ts_") or op == "inst_tvr":

                    alpha_set += self.ts_factory(op, field)

                elif op.startswith("vector"):

                    alpha_set += self.vector_factory(op, field)

                elif op == "signed_power":

                    alpha = "%s(%s, 2)" % (op, field)
                    alpha_set.append(alpha)

                else:
                    alpha = "%s(%s)" % (op, field)
                    alpha_set.append(alpha)

        return alpha_set
