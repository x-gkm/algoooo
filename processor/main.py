import json
import math
import pandas as pd
from collections import Counter, defaultdict


def get_expected_score(rating_a, rating_b):
    return 1 / (1 + 10 ** ((rating_b - rating_a) / 400))


def get_dynamic_k(bounds, submission_count):
    k_min, k_max = bounds
    n0 = 40.0
    c = 0.1
    power = c * (submission_count - n0)
    if power > 100:
        return k_min
    return k_min + (k_max - k_min) / (1 + math.exp(power))


with open("cp_test_data.json", "r", encoding="utf-8") as f:
    data = json.load(f)

user_sub_count = Counter()
problem_sub_count = Counter()

users_elo = {}
problems_elo = {}

K_user_bounds = (32, 80)
K_problem_bounds = (64, 160)

for contest in data:

    user_deltas = defaultdict(float)
    problem_deltas = defaultdict(float)

    for sub in contest["submissions"]:
        user_id = sub["user_id"]
        problem_id = sub["problem_id"]
        is_solved = sub["is_solved"]

        if problem_id not in problems_elo:
            problems_elo[problem_id] = 1500.0
        if user_id not in users_elo:
            users_elo[user_id] = 1500.0

        R_U = users_elo[user_id]
        R_P = problems_elo[problem_id]

        E_U = get_expected_score(R_U, R_P)
        E_P = 1 - E_U

        S_U = is_solved
        S_P = 1 - is_solved

        K_user = get_dynamic_k(K_user_bounds, user_sub_count[user_id])
        K_problem = get_dynamic_k(K_problem_bounds, problem_sub_count[problem_id])

        user_deltas[user_id] += K_user * (S_U - E_U)
        problem_deltas[problem_id] += K_problem * (S_P - E_P)

        user_sub_count[user_id] += 1
        problem_sub_count[problem_id] += 1

    for u_id, delta in user_deltas.items():
        users_elo[u_id] += delta

    for p_id, delta in problem_deltas.items():
        problems_elo[p_id] += delta


df_users = pd.DataFrame(list(users_elo.items()), columns=["User_ID", "Elo"])
df_users["Elo"] = df_users["Elo"].round(2)
df_users = df_users.sort_values(by="Elo", ascending=False)
df_users.to_csv("users.csv", index=False)

df_problems = pd.DataFrame(list(problems_elo.items()), columns=["Problem_ID", "Elo"])
df_problems["Elo"] = df_problems["Elo"].round(2)
df_problems = df_problems.sort_values(by="Elo", ascending=False)
df_problems.to_csv("problems.csv", index=False)
