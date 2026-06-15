import json
import random
import sys


def generate_cp_data():

    users = [
        {"user_id": "tourist", "true_skill": 3500},
        {"user_id": "jiangly", "true_skill": 3300},
        {"user_id": "Benq", "true_skill": 3100},
        {"user_id": "Um_nik", "true_skill": 2900},
        {"user_id": "x-gkm", "true_skill": 2700},
        {"user_id": "Errichto", "true_skill": 2500},
        {"user_id": "NeuralG", "true_skill": 2300},
        {"user_id": "Neal", "true_skill": 2100},
        {"user_id": "Ahmet", "true_skill": 1800},
        {"user_id": "Mehmet", "true_skill": 1600},
        {"user_id": "Can", "true_skill": 1500},
        {"user_id": "Ayşe", "true_skill": 1400},
        {"user_id": "Fatma", "true_skill": 1300},
        {"user_id": "Burak", "true_skill": 1200},
        {"user_id": "Emre", "true_skill": 1100},
        {"user_id": "Deniz", "true_skill": 1000},
        {"user_id": "Ali", "true_skill": 900},
        {"user_id": "Veli", "true_skill": 800},
        {"user_id": "Hasan", "true_skill": 700},
        {"user_id": "Hüseyin", "true_skill": 600},
    ]

    contests_data = []

    for c_id in range(1, 10001):
        contest_name = f"Contest_{c_id}"

        problems = [
            {"problem_id": f"{contest_name}_P{i}", "true_diff": 500 + 300 * i}
            for i in range(0, 10)
        ]

        submissions = []

        for user in users:

            for prob in problems:

                win_probability = 1 / (
                    1 + 10 ** ((prob["true_diff"] - user["true_skill"]) / 400)
                )

                is_solved = 1 if random.random() < win_probability else 0

                submissions.append(
                    {
                        "user_id": user["user_id"],
                        "problem_id": prob["problem_id"],
                        "is_solved": is_solved,
                    }
                )

        contests_data.append({"contest_id": contest_name, "submissions": submissions})

    return contests_data


dataset = generate_cp_data()

json.dump(dataset, sys.stdout, indent=2)
