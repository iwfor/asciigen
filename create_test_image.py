#!/usr/bin/env python3
from PIL import Image, ImageDraw

# Create a simple test image
img = Image.new('RGB', (100, 100), color='white')
draw = ImageDraw.Draw(img)

# Draw a simple pattern
draw.rectangle([10, 10, 50, 50], fill='black')
draw.ellipse([60, 60, 90, 90], fill='gray')
draw.line([(0, 0), (100, 100)], fill='black', width=3)

img.save('test_image.png')
print("Created test_image.png")