import colorsys

a = 'Have pride <3'
s = ''
cols = []
for i, c in enumerate(a):
    s += '%c'
    s += c
    hue = i / (len(a))
    r, g, b = colorsys.hsv_to_rgb(hue, 1.0, 1.0)
    cols.append(f"'color: rgb({r*255},{g*255},{b*255});'");

print(f"console.log('{s}',{','.join(cols)});")
