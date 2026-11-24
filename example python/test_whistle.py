import sys, cactus
wav = sys.argv[1] if len(sys.argv) > 1 else "test.wav"
r = cactus.transcribe(wav, language="fr", word_timestamps=True)
print(r)
