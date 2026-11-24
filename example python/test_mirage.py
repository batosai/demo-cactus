import sys, json, cactus
img = sys.argv[1] if len(sys.argv) > 1 else "test.jpg"
vision = cactus.Mirage()
r = vision.describe(img, sections=["objects", "text"], fields=["bbox"])
print(json.dumps(r, indent=2, ensure_ascii=False, default=str)[:2500])
print(vision.extract(img, {"animal": {"enum": ["dog", "cat", "bird", "fish"]}}))
