# On-chain журнал демо-сценария (devnet)

Program: `CNbAp4fpVhCP6TQ1uPkJv5VZHVZbffM2gvyVXFNJXYbX`

Bond series: `CXoUecbsiGTGQybK5bioHr75k1drjbCAjiWJf58aLHbA`

| # | Действие | Детали | Транзакция |
|---|---|---|---|
| 1 | Выпуск облигации | номинал $1000, купон 10%, 2 раза в год, погашение через 2 года | [65mbJVHt…](https://explorer.solana.com/tx/65mbJVHtNLcTmc9cU43DMdQXT2wLZEWTkZf1kt5vCnMdhsTccWf3quHaMad4asjFXpgaL1UXy1PxG5wFx9wJr7dE?cluster=devnet) |
| 2 | Выдача облигаций: Alice | 10 шт. | [5B5wmkJw…](https://explorer.solana.com/tx/5B5wmkJwhCmeUd88xBMFEnFnwDWBwETvaSWNfhn4QWJy9e3NJZaAXXYxzBiMJXFe9X7MwhkrhLuFsAHiisajT9CQ?cluster=devnet) |
| 3 | Выдача облигаций: Bob | 5 шт. | [ZLHkE3z8…](https://explorer.solana.com/tx/ZLHkE3z8sdugMEFWKACUSYJ2g815ShGcB8xaSo13kfHsk76bxVtrk9Lzdiph1Xifnzb6MhTn3Y79dz719p5mQ2F?cluster=devnet) |
| 4 | Выдача облигаций: Carol | 3 шт. | [65JSGXky…](https://explorer.solana.com/tx/65JSGXkyY6RUYozjJHg8oWrGAjiCwXoY35EFG2CjUT8LaBNxSbvARfDKZ8CwHL2xc7Mcpn5epngdtxmoNvq52jTL?cluster=devnet) |
| 5 | Перевод между холдерами | Alice передаёт Carol 2 облигации (баланс будет 8 / 5 / 5) | [4R6Tjvjt…](https://explorer.solana.com/tx/4R6TjvjtdywFjxBtvqNgb7oVSk4eFeeu46KeqjTL7UH2NcWPRiQqSHf7fwh4NNF9MHrjDEruS4UM6u5jwEvwVw6h?cluster=devnet) |
| 6 | Открытие купонного периода №1 | record date через 182 дня | [AmXVvKe1…](https://explorer.solana.com/tx/AmXVvKe1RZD5i4MqqbbLYjuruvKc9iwx5s37qM21SkxNVZfkRaFtkVUPocSTk92xjZrYz6P4NAyQ6ca644GFP8R?cluster=devnet) |
| 7 | Перемотка времени (демо) | +182 дня | [28FpY8Ek…](https://explorer.solana.com/tx/28FpY8EkxCnWP8AMQZdz8kY6QQBEQcKHyPQ9VZLfXXEBSSRQVFxzksvkwa6ao4gD94rCzzd5NbG41AHsSmFBcic1?cluster=devnet) |
| 8 | Заморозка реестра: Alice | 8 шт. на record date | [5dKjdaWz…](https://explorer.solana.com/tx/5dKjdaWz6Lon2kD8gHYNeT8136SmzZs96F4LtrjR5ksKb1WraRjpraYeGVsqYx8RqEb29va8GjKJ6VKzKgNf8YnX?cluster=devnet) |
| 9 | Заморозка реестра: Bob | 5 шт. на record date | [5GaR7yFQ…](https://explorer.solana.com/tx/5GaR7yFQr4VTKqo2LV8bvLENpjiXH81fGbWKem9GrUTUAGaAhmWUcDq2THYagXg2WLSPYr4EWqu1H9Em6JYnkZki?cluster=devnet) |
| 10 | Заморозка реестра: Carol | 5 шт. на record date | [2WTXSVfJ…](https://explorer.solana.com/tx/2WTXSVfJnoc4B7fKfqtEuEojeZjpfRyHgFygVWPLxFaBuqrSRF1713ep69HH2jVidLER4KT6eG7Zskrkab47ydzN?cluster=devnet) |
| 11 | Фиксация реестра | замороженная сумма равна выпуску, считаем total_due | [5XZP6qjq…](https://explorer.solana.com/tx/5XZP6qjq1rgbMK1g8C6FxC5k5KjAa1fCPY7iZ7EPjsLsK5v5sb8N1NdKreRZGG2y4uvDBX3E6fueKfJ4NXemgV1a?cluster=devnet) |
| 12 | Пополнение vault | $5400: купон + частичное погашение | [269PgvpE…](https://explorer.solana.com/tx/269PgvpEAfWTHGhf62mkjrebM1FuU8s6Njp1Y4zNx12vzuLsLuMPbvSsT7LLhpgwCK9FpuRv3KAmR4yKJruXKR8i?cluster=devnet) |
| 13 | Купон №1: Alice | 8 × $50 = $400 | [54Boautm…](https://explorer.solana.com/tx/54Boautmpzd9vaH4Y3SL5kWW2N2aHfq9EZtqUsuDGiZPrfAewYimqDJBRskqeUBEo9kGAny1c9sSzswjcRWnh3Jx?cluster=devnet) |
| 14 | Купон №1: Bob | 5 × $50 = $250 | [33QRcASC…](https://explorer.solana.com/tx/33QRcASCmugE7hy73CZ9MqduBqpzBLk7HMSBLjFs961JRsBZ2pbWs2zqjp73QTKb73VHBu97aeFZjdjmQbr9nUcj?cluster=devnet) |
| 15 | Купон №1: Carol | 5 × $50 = $250 | [2q2KUmci…](https://explorer.solana.com/tx/2q2KUmciq5oaHnt91eLnvMt75k5D9ifcPctb6VvEYsthmS5zT4fwq51SCHyvuq5GmvNu5sLW8HLQ1rWjZxSEVAha?cluster=devnet) |
| 16 | Объявление частичного погашения | 25% номинала, $250 на облигацию | [5eU8SAxF…](https://explorer.solana.com/tx/5eU8SAxFyZgq6x9GgZFUrab9SPLXjmE2oBWueMrMb8mh5dcGHu5AdPfRZyK8V97oTe6UPtzmt7mP1TQcqo38WLxg?cluster=devnet) |
| 17 | Частичное погашение: Alice | 8 × $250 = $2000 | [3jUcq97w…](https://explorer.solana.com/tx/3jUcq97whV955Fg8V77UMYhnKpZs4uWqA3ukNyie65qa264yTGNcshR14xvVCQBWr6XV5tfCZpDmm7umvW1CFSDr?cluster=devnet) |
| 18 | Частичное погашение: Bob | 5 × $250 = $1250 | [4vygVhvt…](https://explorer.solana.com/tx/4vygVhvtVSuS1ZqB4qu2Xij5HHAaSDWGZNXVVDitszqeSgQh2txRiLuhs63EGPw9utPfjEMuf3fHxAegtYPb8cyp?cluster=devnet) |
| 19 | Частичное погашение: Carol | 5 × $250 = $1250 | [vqUAt5xe…](https://explorer.solana.com/tx/vqUAt5xexfUkNrbvTSWi57PWDyYe8TTTwq8RnyRu6UgaKeqXQ1LL4hA12GCFZZb64kQPbnJJEu2AGz7Q1ThPrZy?cluster=devnet) |
| 20 | Применение частичного погашения | номинал $1000 → $750 | [5jzXsGqQ…](https://explorer.solana.com/tx/5jzXsGqQu3MkrNQfeZVvm3N97GfTxGVRP5CyMFHc25zoEBHGT1qm88HTcvXzap66xL3jjztSXFVDh97zLG3hXdw7?cluster=devnet) |
| 21 | Разморозка: Alice | выплата получена, счёт снова свободен | [3HoTSy2S…](https://explorer.solana.com/tx/3HoTSy2SGdEgAPETcTYgqZ2GeSKn86XqTNBm1Z4imMKNrrw4pVqrh9hJFpZqdA1sbJnRxyUGPWS6NmhFKffqmYCC?cluster=devnet) |
| 22 | Разморозка: Bob | выплата получена, счёт снова свободен | [2Pg1qhTx…](https://explorer.solana.com/tx/2Pg1qhTxA8tYaRJUk8Di4d4SRwZCWxvq6M6Z7Mgyi1CfZ34BshaF1CPBUVD6pdZRoBnuQGa39STxdjLEsWwxULMs?cluster=devnet) |
| 23 | Разморозка: Carol | выплата получена, счёт снова свободен | [2gKyskRv…](https://explorer.solana.com/tx/2gKyskRvKL1REk2qfj6ocYyBUZoFitqCxK56Ubm5TWbQRGBzKn3vgjgRVrtD6rJ5UmLihnWL7kiaLT8hpVqgGaEh?cluster=devnet) |
| 24 | Открытие купонного периода №2 (финальный) | record date = дата погашения | [29HXsGcd…](https://explorer.solana.com/tx/29HXsGcduE3y8rK3LdiPLudnbqXHZTJeH3t9VAfCVkamKD4STnhQivNvWyVDGRDuvTo84tpip95FjeU3WyTveBkE?cluster=devnet) |
| 25 | Перемотка времени (демо) | +548 дней до даты погашения | [38PBsfuZ…](https://explorer.solana.com/tx/38PBsfuZ9zvyeT4vBtincRidc5csjgyWNP2vZ2DY1oKeomBJDo4UZiN8auuYPpYw9xRmsettncUez6Aj7VDSNXZz?cluster=devnet) |
| 26 | Заморозка реестра: Alice | 8 шт. на record date | [3Fdq4XGh…](https://explorer.solana.com/tx/3Fdq4XGhvXfW7pPLB17Akfe7TatqhcWsyUcwRS8ryRJYmepCfJP3TzWnhievgvEzhqBh6z9nc3dqn1hNpV9pY9jZ?cluster=devnet) |
| 27 | Заморозка реестра: Bob | 5 шт. на record date | [3XiCmsNL…](https://explorer.solana.com/tx/3XiCmsNLnEuxJVSH6nmPZEdxUiVf2ToEZUJbCi8CMhYgtFTFzUUUmj41Kut9UM6BSsdPVW43veF9ho6T8apbTgZ2?cluster=devnet) |
| 28 | Заморозка реестра: Carol | 5 шт. на record date | [3uoFTnu4…](https://explorer.solana.com/tx/3uoFTnu4Ni4Eu6qzRZUAx6W8MkNwKcJMLfQkE56P2MNBkaT1mBjHqyW8Y6QUCZ2otzyFrELzFgH674SpuwQ1a3Wn?cluster=devnet) |
| 29 | Фиксация реестра | замороженная сумма равна выпуску, считаем total_due | [4Nf7ZMsX…](https://explorer.solana.com/tx/4Nf7ZMsXSBxcsLuKpJjo6KEv4yg7cfbVHExm4QfZLktxnem6eRTryhSiz1jqxdKYGNsdKUe48pUS3rE5r1eNdq4A?cluster=devnet) |
| 30 | Пополнение vault | $14175: финальный купон + номинал к погашению | [3FHLb7Ji…](https://explorer.solana.com/tx/3FHLb7JiVAy6MdkDi8Evn4ugUM9EfrqHwHHxni167YWADKBgUhTQPdYG46jHgtt6qVeXcgMC1pqSyKd9NZWCeZiX?cluster=devnet) |
| 31 | Купон №2: Alice | 8 × $37.5 = $300 | [49J8nyda…](https://explorer.solana.com/tx/49J8nydaiFJfDihH7gsGzAbkf9p43h5t6bER9VjZ3sYKNnXB4abS6GGFwsaYS7TjrhdJdZqqygRyn6Rpf2Rf858p?cluster=devnet) |
| 32 | Купон №2: Bob | 5 × $37.5 = $187.5 | [3VwpyHJh…](https://explorer.solana.com/tx/3VwpyHJhvNdVJinqixQD6KSj9bPewb9Yo1naa72kbBPk72XWeEFsB3TiH8oy9CX9M45FaY5PmWuoknZWx3dT2b5D?cluster=devnet) |
| 33 | Купон №2: Carol | 5 × $37.5 = $187.5 | [2znB7fji…](https://explorer.solana.com/tx/2znB7fjiJJmF3ETyCirkpsPugtMK339YKAyFiJTg7sVrWbnW9ehnVHkS6GGvjPacnyvebjbk5xdBG8Mzka27WvZF?cluster=devnet) |
| 34 | Разморозка: Alice | выплата получена, счёт снова свободен | [557Xu4uL…](https://explorer.solana.com/tx/557Xu4uLHJSYktvhw57faUyyGRL7TxNujpodqHUkJySB6qoRDECDsCehJNHs6ENTGqmcZDQxejm4V1yKAtafCp31?cluster=devnet) |
| 35 | Разморозка: Bob | выплата получена, счёт снова свободен | [2wHVSHuQ…](https://explorer.solana.com/tx/2wHVSHuQG5c7qhHaoVnN44qa55zrqo2QVFMJm6qmTUbTKJrrxvaiB5GkFPv84zsaVyf9SSDUYzD79BLLNwGYLJFw?cluster=devnet) |
| 36 | Разморозка: Carol | выплата получена, счёт снова свободен | [3C5G8WBg…](https://explorer.solana.com/tx/3C5G8WBgGJSJfKKmrV5ALTfu5hbY4jqsYE6pzmWS8ywdwMLVZpr5ZpZRzgyqYtFnksnZV2whpqaW4FpRhAYKVckx?cluster=devnet) |
| 37 | Погашение: Alice | 8 × $750 = $6000, облигации сожжены | [3LZEkMaS…](https://explorer.solana.com/tx/3LZEkMaSjCas1aZFCgG58sRjQGQZsvvBNSHCyiqv9nLqBK8oMAgHh8jq9LdoeBHFEgKfNxWkrCGbA2vMd92iZXfA?cluster=devnet) |
| 38 | Погашение: Bob | 5 × $750 = $3750, облигации сожжены | [2y1kCsC2…](https://explorer.solana.com/tx/2y1kCsC2QgUekHHTCycYbnxwDV2z6PD5CTeLaZJaotuMhRifJzA91CybjaS3VvucZfgGt45bW5nYieAtnUp4ifwv?cluster=devnet) |
| 39 | Погашение: Carol | 5 × $750 = $3750, облигации сожжены | [2SPQT2Vf…](https://explorer.solana.com/tx/2SPQT2VfEqofwPQjzcdwbwuTaVZbt9nEQxh9KvrWdhWQFxcpSpbWyBqG7KQmTrjTWQwufGmKgf3ELjWzBD1F8K3N?cluster=devnet) |
