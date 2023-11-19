from PIL import Image

j = []

names = ["TERRAN", "JUNGLE", "ROCK", "OCEAN", "DESERT", "ARCTIC", "GAS", "INFERNO", "TOXIC"]
def extract_sprites(image_path, sprite_size, output_path):
    image = Image.open(image_path)
    width, height = image.size

    for sy, name in enumerate(names):
        sprite_count = 0

        if sy >= 7:
            mx = 4
        else:
            mx = 6
        for sx in range(0, mx):
            y = sy * 37 + 71
            x = sx * 37 + 77
            sprite = image.crop((x, y, x + sprite_size, y + sprite_size))
            sprite = sprite.convert("RGBA")
            data = sprite.getdata()
            new_data = [(r, g, b, 0) if r == g == b == 0 else (r, g, b, 255) for r, g, b, _ in data]
            sprite.putdata(new_data)

            q = f"{output_path}/sprite_{name}_{sprite_count}.png"
            j.append(q)
            sprite.save(q)
            sprite_count += 1

if __name__ == "__main__":
    image_path = "assets/PixelPlanets.png"  # Replace with your image path
    sprite_size = 32
    output_path = "assets/"  # Replace with your desired output directory

    extract_sprites(image_path, sprite_size, output_path)
    print(j)
