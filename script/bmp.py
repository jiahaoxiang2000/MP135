from PIL import Image

def create_bmp(filename, width=800, height=480, color=(255, 255, 0, 255)):
    image = Image.new('RGBA', (width, height), color)
    image.save(filename, 'BMP')

if __name__ == "__main__":
    create_bmp('800x480_32bit.bmp')
    print("BMP file '800x480_32bit.bmp' created successfully.")