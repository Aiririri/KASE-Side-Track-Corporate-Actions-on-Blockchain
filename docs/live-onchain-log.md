# On-chain log: live series (devnet)

Program: `CNbAp4fpVhCP6TQ1uPkJv5VZHVZbffM2gvyVXFNJXYbX`

Bond series: `CRuUe5r87C86NFNXwERwMSPebN3dzCQZNuBGk6ui7QH6`

| # | Action | Details | Transaction |
|---|---|---|---|
| 1 | Issue bond | face value $1000, coupon 10%, semi-annual, matures in 2 years | [4jnwuPCj…](https://explorer.solana.com/tx/4jnwuPCjqX3kM8HozFViibSWXsYisqCy5xWvegkRQ8UxJVt8PjAJBF8kVns1yhKKUwoWz4xCSm6Bz8PCfF2AFYDS?cluster=devnet) |
| 2 | Issue bonds: Alice | 10 bonds | [3YXRP3ST…](https://explorer.solana.com/tx/3YXRP3ST3WQxcRLVbuXW6zRMAmfCeaXGvRS3mhUxU3orKV8iRp5oYmT583F3CMrHKBFngxALPKVWYEEUp6Mew8Jv?cluster=devnet) |
| 3 | Issue bonds: Bob | 5 bonds | [3GMhW46n…](https://explorer.solana.com/tx/3GMhW46nA8nRbJTHbxnpvNrn9mBUzikvzfAkFdak6ebu4j8BL71j6eVazZrRw4P2V4JKNqnjrZFVEzExV3iHk6a5?cluster=devnet) |
| 4 | Issue bonds: You (Phantom) | 4 bonds | [5xyGyrkU…](https://explorer.solana.com/tx/5xyGyrkUnePRVsc6kS9xYXVFqpjvUbeA87vYCT4RRF5h2kgRDHptwT1rU3PYuEjZrpQ17raLUvv8Kf6SppRtnJrG?cluster=devnet) |
| 5 | Transfer between holders | Alice sends 2 bonds to You (balances become 8 / 5 / 6) | [5oLrEbjS…](https://explorer.solana.com/tx/5oLrEbjSLmLdMHgawg6UBCkGdRstiN3gDy6tR22pALQyu7w9mR2E4kDtSbehfcqqfsB9DiCjxo95SRQrS2Hfe72G?cluster=devnet) |
| 6 | Open coupon round #1 | record date in 182 days | [gKwoy8MS…](https://explorer.solana.com/tx/gKwoy8MS99JaPKfpKJvSWcbhCgQo5DFjfZQptwAq56d6ruGbCFqGwGwsxodyDjMqDXJawt4x7X6urezRRvgzurY?cluster=devnet) |
| 7 | Advance demo clock | +182 days | [2mMBdmei…](https://explorer.solana.com/tx/2mMBdmeio3fTrWvf4mYNaQGNo8QSfM6v4wXnLF981pU55iFo7QuRUJUs8ogdvXmJzdJurrzjuU1N97Wfwb3Rw7oJ?cluster=devnet) |
| 8 | Freeze holder: Alice | 8 bonds at record date | [5UQtzies…](https://explorer.solana.com/tx/5UQtziesyixSVth9Rmq7a5UkV8ryd7AoYj9882TtVDcj4wkKhW9Q4ELgFDUo6DgP1MmVo9fRqXF4mapLeSd7qFE6?cluster=devnet) |
| 9 | Freeze holder: Bob | 5 bonds at record date | [5GXuKfex…](https://explorer.solana.com/tx/5GXuKfexKVa7LAkhvpuSEDpUUbPrBk8n8QPEn1763UrHgVKiUGBVs4zemK5sNrez5Yka4YX2bYpkZ4Vzqs93VRQ?cluster=devnet) |
| 10 | Freeze holder: You (Phantom) | 6 bonds at record date | [3GEH57Q2…](https://explorer.solana.com/tx/3GEH57Q2PLrB5kb5s4ndsofemM9uuZ87jt3d2XFF78cDVstA8zg7zMiHH5WnHF5s1S9eBqt7EXmVpAkQpG9Ni3Se?cluster=devnet) |
| 11 | Finalize record date | frozen supply equals total supply; total due is fixed | [31gVGpgb…](https://explorer.solana.com/tx/31gVGpgbJY3N5bcWMQYKyabAJEM3Da76UMCUU7yaNgNNyoY4uFYLWaQ68MV9pb8hVyo6jaRBar6aT5a6A9jHwdDF?cluster=devnet) |
| 12 | Fund vault | $5700: coupon + partial redemption | [WYnfP4fv…](https://explorer.solana.com/tx/WYnfP4fvgK87RohrujcdsiJwQ3pQAibtRWA54WSWbsjr2U5aJKxbwTcAKGpSXZoHFzW7PFSAqtTPcBQtbE6aDff?cluster=devnet) |
| 13 | Coupon #1: Alice | 8 × $50 = $400 | [293kjx62…](https://explorer.solana.com/tx/293kjx625ooMXnczYwJSgUhx8vAmoj5dUa65ggWequQETDCQnzbhrmGzaprZPxSQkhku2K6sHeNjNJWoA1jBAs6P?cluster=devnet) |
| 14 | Coupon #1: Bob | 5 × $50 = $250 | [2TZCJv4W…](https://explorer.solana.com/tx/2TZCJv4WostvMXebN2RLx47dVVnAWgLvtTR88GuAzBxQWGWc8u4pd9NmqEEneYY4SgaQvdnNh9YaSJA3wMQBTX4F?cluster=devnet) |
| 15 | Coupon #1: You (Phantom) | 6 × $50 = $300 | [ZBJSWMo5…](https://explorer.solana.com/tx/ZBJSWMo5avBezhVwd3XMufFQccgxzgYSP4rpRTXvJ6MMqHC2PCR9vjG7CabFy7kp5fnetRjAh4xLLrXBTcSyuP7?cluster=devnet) |
| 16 | Schedule partial redemption | 25% of face value, $250 per bond | [24cEx4XG…](https://explorer.solana.com/tx/24cEx4XGcXtSTbtSDojJsJxhZgNs6Ar7npoDockf7g1SKDnc5ZdBrSG3RCQJfxQZXo7FRPPWD2Gvf7mzv8LkVJBn?cluster=devnet) |
| 17 | Partial redemption: Alice | 8 × $250 = $2000 | [2c6vhHTn…](https://explorer.solana.com/tx/2c6vhHTnbfxSVFErRV63wRDeF1cEDBxXPG2iRfCdNuCJKeAj8dihDUahGaL2u9h7iDa7Q9j3hjVeCAUmaHSguXmk?cluster=devnet) |
| 18 | Partial redemption: Bob | 5 × $250 = $1250 | [4pmQk2kz…](https://explorer.solana.com/tx/4pmQk2kzYFJkS5aUB44r3AA7LpdyeaMpYm8tqNW7QpHuQW9h754JvvDSTT5zAKJPB1etqeeMLFvnnKnCiT9dPBc8?cluster=devnet) |
| 19 | Partial redemption: You (Phantom) | 6 × $250 = $1500 | [v41JAcY5…](https://explorer.solana.com/tx/v41JAcY5tUvjLMxg1P51byazYssoA62Xsnbw1RqCZiAMXnCYruDxiFCnjUzMiwe9kcGoKULo6XoRK2RnH7L6Tcx?cluster=devnet) |
| 20 | Apply partial redemption | face value $1000 → $750 | [3e2S2LWk…](https://explorer.solana.com/tx/3e2S2LWkKMjmY9yT8C3WGkv4Dv2RkqCHWaiwGtaQs4MM3XGspCNoz1v2Cdx9haTuuFZrYK1bnnWYqtcmKqsnxJEB?cluster=devnet) |
| 21 | Thaw holder: Alice | payout received, account released | [5R3eT2vG…](https://explorer.solana.com/tx/5R3eT2vG7WsGKRQW7YDRQJ8ems4c4bKAE2MYzVyD69CA4r6CB3AExDHEeCEsnimDeTfYZj2yVJrbFDYasxsgcAuT?cluster=devnet) |
| 22 | Thaw holder: Bob | payout received, account released | [3NwktbcF…](https://explorer.solana.com/tx/3NwktbcFZDYhKJiM2HRSWPddTajPXQrLhpgho3mB4VcACUPhLXALERBzznh7kA249AsFXtJWsSgBH1YG1H5MAX8R?cluster=devnet) |
| 23 | Thaw holder: You (Phantom) | payout received, account released | [K6hfz4Pp…](https://explorer.solana.com/tx/K6hfz4PpRj2Wwdw4QYTNenVD7ta57xVUF1Bq8krufqMvLixknqnC8w3x97V7YpqUiKJWhaStC4YnvJVFY48Prf4?cluster=devnet) |
| 24 | Open coupon round #2 (final) | record date = maturity | [5hLjjQox…](https://explorer.solana.com/tx/5hLjjQoxRawTzuhSwUzzEUvX2hqUgwLjhoWnJvkByegTUfoWKMBizwyGgnh4y7pygndg5dduP88LjMrMe593ZVGD?cluster=devnet) |
| 25 | Advance demo clock | +548 days to maturity | [5hPzNyMr…](https://explorer.solana.com/tx/5hPzNyMrgTrqwRDwrZUSRf5iR1bAJtD8Mj4P4TDHPMGMdfGviAAJSN9ZARMQTcCkhFbKLzQYC18atX3nHu38w6rP?cluster=devnet) |
| 26 | Freeze holder: Alice | 8 bonds at record date | [29d5gnAq…](https://explorer.solana.com/tx/29d5gnAqgSrBqEB7WVW2tu1FbHx6993JyjMfrFgWR8FeugF3W2G4a3SCYXmtYkkrbCr7suicpMyXkoSXmHX7BX1c?cluster=devnet) |
| 27 | Freeze holder: Bob | 5 bonds at record date | [v37aJiUS…](https://explorer.solana.com/tx/v37aJiUS88iBLSfDPdbpBV1DP6si74X8pxJofw9hZjH5HHCjzWLjimA5wfx8DWuoHHiwDAYc67ttudnq7LihLhN?cluster=devnet) |
| 28 | Freeze holder: You (Phantom) | 6 bonds at record date | [deBnW3Tq…](https://explorer.solana.com/tx/deBnW3TqHXvcPkChjRDgxJWuxDUtGBjwcd6CtaZVWTUmsL4wTLvEKt8Z2zww6U3ByoNPcga3zUaocJmN6JXZx5b?cluster=devnet) |
| 29 | Finalize record date | frozen supply equals total supply; total due is fixed | [3nQ7ivZR…](https://explorer.solana.com/tx/3nQ7ivZRSQ3sykBEzAFULXijAvPKk7LWCCPgV86ZM2Eu4QGy93q96GKdWgnYF2BrL1bYVUQBURjsGYhz7N2yeaYy?cluster=devnet) |
| 30 | Fund vault | $14962.5: final coupon + principal for all holders | [4cXkRM9F…](https://explorer.solana.com/tx/4cXkRM9F6wNcNiCYikCSbC6am51cdMiBW53cDDnag8bdFQ4xthdu6ZVpQsmjUkMaDY6eU8zzG1roj6BYUNyD6taX?cluster=devnet) |
| 31 | Coupon #2: Alice | 8 × $37.5 = $300 | [ZH3YNbEf…](https://explorer.solana.com/tx/ZH3YNbEfgyMYvjhRQJc3Yqc1SPuQYExNwdwktiReMi1kss3cGK3nD71gowPPNTByFYhUKRQ1r2hFUR7YUpKbKsY?cluster=devnet) |
| 32 | Coupon #2: Bob | 5 × $37.5 = $187.5 | [3Z25wPxz…](https://explorer.solana.com/tx/3Z25wPxzJTEqQ2y3Ncejwsxyxq8qYiDtKYyNMN1yCgLAHRS2Rp27PHNsiFDTJHDpgjB4WYiRQhw8LMSpZW7o3zyZ?cluster=devnet) |
| 33 | Coupon #2: You (Phantom) | 6 × $37.5 = $225 | [43AnUbDh…](https://explorer.solana.com/tx/43AnUbDhN4t4MpyNcxuGtUdxquji6S1JpMHvYL7WDpWF4edjDbX6MayVmADc68SXx7685DD9cr3WG1xd2jKhsLWx?cluster=devnet) |
| 34 | Thaw holder: Alice | payout received, account released | [P4Cdt5vw…](https://explorer.solana.com/tx/P4Cdt5vwEeeccszGuFHtxCmctLYKzbNwVCGfsMAQem1yfJhUS9PjEMDDFpnbFSmj8jzHXoRk5aE5TGbFg3YcTux?cluster=devnet) |
| 35 | Thaw holder: Bob | payout received, account released | [4jV6VYXA…](https://explorer.solana.com/tx/4jV6VYXAX5KFcEkzEgsATWFyTahhEQcM7AzmAdBvqSVuv9jDH8LLs65ETSns6YrWYni9fD9x8s8GAtpeSVQV2wWT?cluster=devnet) |
| 36 | Thaw holder: You (Phantom) | payout received, account released | [4mUfrnZq…](https://explorer.solana.com/tx/4mUfrnZq82GCXoiHPBCkucuMPTtEMBDYxoFiCWL3wow8SLvEgg1m3vsLEio9xm6dHZFoHcFZNrvB2Sv8FJtW9wAt?cluster=devnet) |
| 37 | Redeem: Alice | 8 × $750 = $6000, bonds burned | [5PbTkvDy…](https://explorer.solana.com/tx/5PbTkvDy58uvW6Hx84UF19RMradcWNd7YsDieNaY17NB62zenoJFJefPeoN9w7jW1NZYABjYL5qGov9QHmkSMGZb?cluster=devnet) |
| 38 | Redeem: Bob | 5 × $750 = $3750, bonds burned | [fC1CUimu…](https://explorer.solana.com/tx/fC1CUimui2eMLrBsWjvm1Ec3r7RqUAP8SxdYym52JattVKMWUaYW3Xsu9Ntv6DnmB43WjQqoDBMqMo8dByYfcSM?cluster=devnet) |
