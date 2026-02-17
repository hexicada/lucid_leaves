import os
from PIL import Image

# 1. SETUP
FRAME_WIDTH = 64   # Target width for one frame
FRAME_HEIGHT = 64  # Target height for one frame
OUTPUT_NAME = "beryl_sheet.png"
FOLDER_NAME = "beryl_frames"  # Where your 13 images live

def pack_sheet():
    # 2. FIND IMAGES
    # Get all .png files and sort them (1.png, 2.png, 10.png...)
    files = [f for f in os.listdir(FOLDER_NAME) if f.endswith(".png")]
    
    # Crucial: Sort numerically, not alphabetically (so 10 comes after 9, not 1)
    files.sort(key=lambda f: int(''.join(filter(str.isdigit, f))))
    
    if not files:
        print("No PNGs found!")
        return

    print(f"Found {len(files)} frames. Packing...")

    # 3. CREATE BLANK CANVAS
    # Width = 13 * 64, Height = 64
    sheet_width = FRAME_WIDTH * len(files)
    sheet_height = FRAME_HEIGHT
    
    # "RGBA" means Red, Green, Blue, Alpha (Transparency)
    sheet = Image.new("RGBA", (sheet_width, sheet_height), (0, 0, 0, 0))

    # 4. PASTE THEM IN A ROW
    for index, filename in enumerate(files):
        img_path = os.path.join(FOLDER_NAME, filename)
        img = Image.open(img_path)
        
        # Resize to 64x64 if they aren't already (Optional safety)
        img = img.resize((FRAME_WIDTH, FRAME_HEIGHT), Image.NEAREST)
        
        # Calculate position: (0,0), (64,0), (128,0)...
        x_pos = index * FRAME_WIDTH
        sheet.paste(img, (x_pos, 0))
        
        print(f"Packed {filename} at x={x_pos}")

    # 5. SAVE
    sheet.save(OUTPUT_NAME)
    print(f"Done! Saved to {OUTPUT_NAME}")

if __name__ == "__main__":
    pack_sheet()