from typing import Literal, Optional
import pandas as pd
import re
from tqdm import tqdm

from .logger_util import logger
from .api import BrainApi
from .database import AlphaDao


class BrainApp:
    def __init__(
        self,
        region: str = "USA",
        universe: str = "TOP3000",
        delay: Literal[0, 1] = 1,
        decay: int = 0,
        neutralization: str = "SUBINDUSTRY",
        truncation: float = 0.1,
        pasteurization: Literal["ON", "OFF"] = "ON",
        test_period: str = "P1Y0M0D",
        unit_handling: Literal["VERIFY"] = "VERIFY",
        nan_handling: Literal["ON", "OFF"] = "OFF",
        visualization: bool = True,
        alpha_type: Literal["REGULAR", "SUPER"] = "REGULAR",
        api: Optional[BrainApi] = None,
    ) -> None:
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
        if api:
            self.api: BrainApi = api
        else:
            self.api: BrainApi = BrainApi()

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
                visualization=self.visualization
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

        logger.info(f"Found {len(reused_alpha_ids)} duplicates, {len(new_alpha_list)} new alphas to simulate.")
        return new_alpha_list

    def simulate_alpha(self, alpha_list):
        n = 2
        sim_alphas = [alpha_list[i:i + n] for i in range(0, len(alpha_list), n)]
        with tqdm(total=len(sim_alphas), desc="Simulation Progress") as pbar:
            for index, alphas in enumerate(sim_alphas):
                if index > 0:
                    self.api.check_session_and_relogin()
                result_list = self.api.simulate_alpha_list_multi(alphas)
                for result in result_list:
                    alpha_id = result['alpha_id']
                    regular = result['simulate_data']['regular']
                    filtered_alphas = list(filter(lambda item: item['regular'] == regular, alphas))
                    if len(filtered_alphas) > 0:
                        AlphaDao.add_to_cache(filtered_alphas[0], alpha_id)
                        logger.info(f"simulate completed: {alpha_id}, regular: {regular}")
                    else:
                        logger.warning(f"Cannot found {regular} in: {alphas}")
                pbar.update(1)

    def simulation(self, expression_list: list[str]):
        alpha_list = self.process_alpha(expression_list)
        if len(alpha_list) == 0:
            return
        self.simulate_alpha(alpha_list)

    def get_dataset_id(
        self,
        name: str,
        theme: Optional[bool] = None,
    ) -> Optional[str]:
        datasets_df = self.api.get_datasets(region=self.region, delay=self.delay, universe=self.universe, theme=theme)
        selected_datasets_df = datasets_df[datasets_df["name"].str.contains(name, case=False)]
        if len(selected_datasets_df) == 0:
            return None
        return str(selected_datasets_df.id.values.tolist()[0])

    def get_datafields_by_dataset_id(
        self,
        dataset_id: str,
        type: Literal["MATRIX", "VECTOR"] = "MATRIX",
        search: str = ""
    ) -> pd.DataFrame:
        datafields_df = self.api.get_datafields(region=self.region, delay=self.delay, universe=self.universe, search=search)
        filtered_datafields_df = datafields_df[
            (datafields_df['dataset_id'] == dataset_id)
            & (datafields_df['type'] == type)
        ].reset_index(drop=True)
        return filtered_datafields_df
    
    def get_datasets_field_list(self, dataset_name: str, theme: Optional[bool] = None, type: Literal["MATRIX", "VECTOR"] = "MATRIX") -> list:
        dataset_id = self.get_dataset_id(dataset_name, theme=theme)
        logger.info(f"dataset_id: {dataset_id}")
        if not dataset_id:
            return []
        datafileds_df = self.get_datafields_by_dataset_id(dataset_id=dataset_id, type=type)
        field_list = datafileds_df.id.values.tolist()
        logger.info(f"total data fields: {len(field_list)}")
        return field_list
    
    def generate_operators_info(self, scope: Literal["REGULAR", "SUPER"] = 'REGULAR') -> str:
        operators = self.api.get_operators()
        operators = operators[operators['scope'] == scope]
        operators_info = operators[['definition', 'category', 'description']]
        md_table = operators_info.to_markdown(index=False)
        clean_md = re.sub(r' +\| +', '|', md_table)
        clean_md = re.sub(r' +\|', '|', clean_md, flags=re.M)
        clean_md = re.sub(r'\| +', '|', clean_md, flags=re.M)
        return clean_md
    
    def generate_datafields_info(self, dataset_name: str, theme: Optional[bool] = None, type: Literal["MATRIX", "VECTOR"] = "MATRIX") -> str:
        dataset_id = self.get_dataset_id(dataset_name, theme=theme)
        if not dataset_id:
            logger.warning(f"dataset_id not found: {dataset_name}")
            return ""
        datafields_df = self.get_datafields_by_dataset_id(dataset_id=dataset_id, type=type)
        datafields_info = datafields_df[['id', 'description', 'type', 'subcategory_name']]
        md_table = datafields_info.to_markdown(index=False)
        clean_md = re.sub(r' +\| +', '|', md_table)
        clean_md = re.sub(r' +\|', '|', clean_md, flags=re.M)
        clean_md = re.sub(r'\| +', '|', clean_md, flags=re.M)
        return clean_md
