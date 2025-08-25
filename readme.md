# Requirements

Rust
Piper
FFMPEG

## Command

cargo run --release -- --subreddit AITAH --background ".\res\bg.mp4" --out out.mp4 --piper-model ".\en_US-amy-medium.onnx"

## Recommended Subreddits

### AITAH

r/AITAH
r/AmITheAsshole
r/AmItheButtface

### Entitlement

r/EntitledPeople
r/ChoosingBeggars

### Confessions

r/offmychest
r/TrueOffMyChest
r/Confessions

### Relationship Advice

r/RelationshipAdvice
r/relationships
r/dating_advice

### Drama

r/MaliciousCompliance
r/ProRevenge
r/NuclearRevenge
r/PettyRevenge
r/TalesFromYourServer
r/TalesFromTechSupport

### Funny / Relatable

r/TodayIF*ckedUp
r/TooAfraidToAsk

### Work / Life Stories

r/AntiWork
r/WorkReform
r/Teachers

## Future Subreddits

### AskReddit

r/AskReddit
r/AskMen
r/AskWomen

## Run Command

cargo run --release -- --subreddit AITAH --background ".\res\bg.mp4" --out out.mp4 --piper-model ".\tts\en_US-hfc_male-medium.onnx"
