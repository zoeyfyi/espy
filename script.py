import Xlib
import Xlib.display

### Create virtual display and open the browser here ###

dpy = Xlib.display.Display(":73")
root = dpy.screen().root
print("root", root)
geometry = root.get_geometry()
print("geometry", geometry)
for win in root.query_tree().children:
    print("win", win, win.id)
    win.configure(x = 0, y = 0,
        width = geometry.width, height = geometry.height)
dpy.sync()
