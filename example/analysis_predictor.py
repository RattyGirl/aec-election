import math

import numpy
import psycopg2
import matplotlib
import matplotlib.pyplot as plt
from numpy import polynomial
import string

matplotlib.use('Qt5Agg')
import numpy as np
from builtins import print, list, map, float, len, int
from scipy.stats import t

# cursor = connection.cursor()
#
# cursor.execute("SELECT * from portal.portal_users;")
#
# # Fetch all rows from database
# record = cursor.fetchall()
#
# print("Data from Database:- ", record)
election_2016 = "20499"
election_2019 = "24310"
election_2022 = "27966"
last_id = election_2019
cur_id = election_2022


def get_divisions(conn, event_id) -> list:
    cursor = conn.cursor()
    cursor.execute("SELECT * from contest WHERE event_id = %s AND identifier = %s;", (event_id, "H"))
    records = cursor.fetchall()
    out = []
    for row in records:
        out.append((row[2], row[4]))
    return out


class Booth:
    def __init__(self, timestamp, id, vote, total, historical, historical_total):
        self.timestamp = timestamp
        self.id = id
        self.vote = vote
        self.total = total
        self.historical = historical
        self.historical_total = historical_total

    def tostring(self):
        return f"{self.id}, {self.vote}, {self.total}, {self.historical}, {self.historical_total}"


def get_all_necessary_data(conn, electorate_id, party):
    cursor = conn.cursor()
    query = f"""SELECT * from resultstotal cur
    LEFT JOIN (
        SELECT * from resultstotal current
            WHERE event_id= '{last_id}' AND (affiliation='{party}') ) past ON cur.contest_id = 
            past.contest_id AND cur.polling_place_id = past.polling_place_id AND cur.affiliation = past.affiliation 
            WHERE cur.event_id= '{cur_id}' AND (cur.affiliation='{party}') AND cur.contest_id = '{electorate_id}';"""
    cursor.execute(query)
    records = cursor.fetchall()
    return list(map(lambda row: Booth(row[7], row[2], row[5], row[6], row[13], row[14]), records))


# def determine_bias(vote, total, enrolment):
#     d_Pi_i = vote / total
#     d_Beta_i = d_Pi_i - last_percentage_total
#     cum_total = np.cumsum(total)
#     z_i = (last_cum_total - (total / 2)) / enrolment
#     # ax.scatter(z_i, d_Beta_i)
#     idx = np.isfinite(z_i) & np.isfinite(d_Beta_i)
#     # polynomial - d_Beta_i (incremental bias)
#     return np.poly1d(numpy.polyfit(z_i[idx], d_Beta_i[idx], deg=3))

def prediction(sorted_booths):
    last_result = 0.4408
    enrolment = 108402
    fig, ax = plt.subplots(num=None, figsize=(16, 12), dpi=80, facecolor='w', edgecolor='k')
    # ignore if nan in either
    sorted_booths = [
        item for item in sorted_booths
        if item.historical_total is not None and not (isinstance(item.historical_total, float) and math.isnan(item.historical_total))
    ]
    cum_total_votes = np.cumsum(np.array(list(map(lambda booth: booth.vote, sorted_booths))))
    cum_total_counted = np.cumsum(np.array(list(map(lambda booth: booth.total, sorted_booths))))
    booth_results = np.array(list(map(lambda booth: booth.vote, sorted_booths)))/np.array(list(map(lambda booth: booth.total, sorted_booths)))
    booth_sizes = np.array(list(map(lambda booth: booth.total, sorted_booths)))

    cum_last_votes = np.cumsum(np.array(list(map(lambda booth: booth.historical, sorted_booths))))
    cum_last_counted = np.cumsum(np.array(list(map(lambda booth: booth.historical_total, sorted_booths))))

    cum_cur_percentage = cum_total_votes/cum_total_counted
    cum_last_percentage = cum_last_votes/cum_last_counted
    cum_swing = cum_cur_percentage-cum_last_percentage
    print(cum_last_votes)
    print(cum_last_counted)
    cur_booth_percentage = np.array(list(map(lambda booth: booth.vote/booth.total if booth.total else 0, sorted_booths)))
    last_booth_percentage = np.array(list(map(lambda booth: booth.historical/booth.historical_total if booth.historical_total else 0, sorted_booths)))
    # print(cum_last_percentage)
    # ax.plot(
    #     cum_total_counted/enrolment,
    #     cum_swing + 0.5,
    #     label="Cumlative swing"
    # )
    ax.plot(
        cum_total_counted/enrolment,
        cum_total_votes/cum_total_counted,
        label="Current - Lib"
    )
    ax.plot(
        cum_last_counted/enrolment,
        cum_last_votes/cum_last_counted,
        label="Last - ALP"
    )
    prediction = cum_swing + last_result
    ax.plot(
        cum_total_counted/enrolment,
        prediction,
        label="Prediction",
        linestyle=":",
        color="black"
    )
    # calculate confidence interval
    min_interval = []
    max_interval = []
    z_score = 1.96 #95%
    z_score = 2.576 #99%
    for i in range(len(booth_results)):
        # Use only the first i booths
        pred = prediction[i]
        sizes = booth_sizes[i]
        std_error = np.sqrt((pred*(1-pred))/enrolment)
        confidence_interval = z_score * std_error
        print(pred)
        min_interval.append(pred-confidence_interval)
        max_interval.append(pred+confidence_interval)
    print(min_interval)
    print(max_interval)

    ax.fill_between(cum_total_counted/enrolment,
                    min_interval,
                    max_interval, color='blue', alpha=0.2, label='95% Confidence Interval')
    # ax.fill_between(cum_means,
    #                 cum_means - conf_interval,
    #                 cum_means + conf_interval,
    #                  color='blue', alpha=0.2, label='95% Confidence Interval')


# vertical line for each booth
    # ax.vlines(x=cum_total_counted/enrolment, ymin=0, ymax=1, linestyle=":", color="black")

    ax.legend()
    ax.set_xlim([0, 1])
    ax.set_ylim([0, 1])
    ax.axhline(y=0.5, color="red")
    ax.axvline(x=0.03, linestyle=":", color="black")
    ax.axvline(x=0.1, linestyle=":", color="black")
    ax.grid(True)
    fig.tight_layout()
    plt.xlabel("% counted")
    plt.ylabel("Green vote")
    plt.title("Wills")
    plt.show()

def main():
    connection = psycopg2.connect(database="aec", user="aec", host="localhost", port=5432)

    # booths: list[Booth] = get_all_necessary_data(connection, 234, "(cur.affiliation='GRN' OR cur.affiliation='GVIC')")
    booths: list[Booth] = get_all_necessary_data(connection, 234, "ALP")
    booths = np.array(booths)
    # for booth in booths:
    #     print(booth.tostring())

    # fig, ax = plt.subplots(num=None, figsize=(16, 12), dpi=80, facecolor='w', edgecolor='k')
    # totals = np.array(list(map(lambda booth: booth.total, booths)))

    booths_sorted = booths[np.argsort(np.array(list(map(lambda booth: booth.timestamp, booths))))]
    prediction(sorted_booths=booths_sorted)
    # cur_vote = np.array(list(map(lambda booth: booth.vote, booths_sorted)))
    # cum_vote = np.cumsum(cur_vote)
    # cur_total = np.array(list(map(lambda booth: booth.total, booths_sorted)))
    # cum_total = np.cumsum(cur_total)
    # cum_percentage = cum_vote / cum_total
    # cur_percentage = cur_vote / cur_total
    #
    # last_totals = np.array(
    #     list(map(lambda booth: booth.historical_total if booth.historical_total else 0, booths_sorted)))
    # last_vote = np.array(list(map(lambda booth: booth.historical if booth.historical else 0, booths_sorted)))
    #
    # last_cum_vote = np.cumsum(last_vote)
    # last_total = np.array(
    #     list(map(lambda booth: booth.historical_total if booth.historical_total else 0, booths_sorted)))
    # last_cum_total = np.cumsum(last_total)
    # last_cum_percentage = last_cum_vote / last_cum_total
    # last_percentage = last_vote / last_total
    #
    # last_percentage_total = last_cum_percentage[-1]
    #
    # # matched prediction
    # matched_swing = cur_percentage - last_percentage
    # matched_swing = np.nan_to_num(matched_swing)
    # n = np.arange(1, len(matched_swing) + 1)
    # cum_mean_m_swing = np.cumsum(matched_swing) / n
    # matched_prediction = last_percentage_total + cum_mean_m_swing
    #
    # # confidence interval
    # enrolment = 114388
    # confidence = 0.95
    # z_score = 1.96
    # std_error = np.sqrt(matched_prediction * (1 - matched_prediction) / cum_total)
    # fpc = np.sqrt((enrolment - n) / (enrolment - 1))
    # margin_of_error = z_score * std_error * fpc
    # # cum_std = np.sqrt(np.cumsum((matched_swing - cum_mean_m_swing) ** 2) / (n - 1))
    # # cum_std[0] = 0
    # #
    # # sem = cum_std / np.sqrt(n)
    # # fpc = np.sqrt((enrolment - n) / (enrolment - 1))
    # # sem_adjusted = sem * fpc
    # #
    # # t_value = t.ppf((1 + confidence) / 2, n - 1)
    # # margin_of_error = t_value * sem_adjusted
    #
    # lower_bound_swing = cum_mean_m_swing - margin_of_error
    # lower_bound_prediction = last_percentage_total + lower_bound_swing
    # upper_bound_swing = cum_mean_m_swing + margin_of_error
    # upper_bound_prediction = last_percentage_total + upper_bound_swing
    #
    # ax.plot(last_cum_total / enrolment, last_cum_percentage, label="Last Real Result", color="blue",
    #         linestyle="-")
    # # ax.scatter(cum_total, grn_vote)
    # # ax.scatter(cum_total / enrolment, matched_prediction, label="Prediction", color="green", linestyle=":")
    # # ax.scatter(last_cum_total / enrolment, (last_cum_vote/last_cum_total) - last_percentage_total, color="black", label="cum bias")
    # bias_regression_model = determine_bias(last_vote, last_totals)
    # # model_range = np.linspace(0, last_cum_total[-1] / enrolment, 200)
    # # model_fit = bias_regression_model(model_range)
    # # hat_Beta_i = np.cumsum(model_fit)/model_range
    # # ax.scatter(last_cum_total / enrolment, (last_cum_vote/last_cum_total) - last_percentage_total, color="black", label="bias by booth")
    # # ax.scatter(cum_total / enrolment, lower_bound_prediction, label="Prediction", color="black", linestyle="-.")
    # # ax.scatter(cum_total / enrolment, upper_bound_prediction, label="Prediction", color="black", linestyle="-.")
    # # use model
    # new_model_fit = bias_regression_model(cum_total/enrolment)
    # new_hat_Beta_i = np.cumsum(new_model_fit)/(cum_total/enrolment)
    # print(cum_percentage-new_hat_Beta_i)
    # ax.scatter(cum_total/enrolment, cum_percentage-new_hat_Beta_i, color="black", label="Prediction")
    # ax.plot(cum_total / enrolment, cum_percentage, label="Real Result", color="green", linestyle="-")
    # # ax.axhline(y=cum_percentage[-1], color="red", linestyle="--")
    # ax.axhline(y=0.5, color="red")
    # ax.axvline(x=last_cum_total[-1] / enrolment)
    #
    # ax.legend()
    # # ax.set_xlim([0, 1])
    # # ax.set_ylim([0, 1])
    #
    # ax.grid(True)
    # fig.tight_layout()
    # plt.xlabel("% counted")
    # plt.ylabel("Green vote")
    # plt.title("Wills")
    # plt.show()

if __name__ == "__main__":
    main()

# SELECT p.event_id,p.contest_id,p.polling_place_id,c.name, c.affiliation,p.votes from primaryvote p LEFT JOIN candidate c ON p.candidate_id=c.id AND p.event_id=c.event_id AND p.contest_id=c.contest_id WHERE p.event_id='27966';
