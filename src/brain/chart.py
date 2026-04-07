import matplotlib.pyplot as plt
import seaborn as sns
from brain.database.is_stats_dao import IsStatsDao


class BrainChart:
    def __init__(self) -> None:
        pass

    def load_data(self, dataset_id: str):
        return IsStatsDao.query_by_dataset_id(dataset_id)

    def plot_histogram(self, y, column_name):
        sns.set_theme(style="whitegrid")
        plt.figure(figsize=(10, 6))
        sns.histplot(y, kde=False, color='skyblue', edgecolor='black')
        plt.xlabel(column_name, fontsize=14)
        plt.ylabel('Frequency', fontsize=14)
        plt.tight_layout()
        plt.show()