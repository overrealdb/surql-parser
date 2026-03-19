/**[test]

[[test.results]]
value = "NONE"

[[test.results]]
value = "NONE"

[[test.results]]
value = "[{ bank_accounts: [55555, 55555, 98787], id: customer:q57ltnsy4rnpwtica0qa, languages: {'en', 'ja', 'kr'} }]"

*/

DEFINE FIELD bank_accounts ON TABLE customer TYPE array<int>;
DEFINE FIELD languages ON TABLE customer TYPE set<string>;

CREATE customer SET
    bank_accounts = [
      55555,
      55555,
      98787
    ],
    languages = <set>[
        "en",
        "ja",
        "kr",
        "kr"
    ];
