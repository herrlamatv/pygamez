//! moduldoku.rs
//! ============
//! Die `__doc__`-Saetze aller Module - **aus echtem pygame erzeugt.**
//!
//! Erzeugt von `tools/gen_moduldoku.py`; nicht von Hand aendern. Warum es
//! diese Tabelle gibt, steht dort: `__doc__` ist Schnittstelle und gehoert
//! damit pygame, waehrend die Doku-Kommentare im Quelltext deutsch
//! bleiben. An einem **Modul** ist `__doc__` schreibbar, an einer Methode
//! nicht - deshalb nur Module.
//!
//! Gemessen fehlte der Satz bei **36** von 60 Modulen ganz
//! (`help(rustgame_py.draw)` zeigte nichts), und bei den eingebetteten
//! Python-Modulen stand deutsche Prosa darin.

/// `(Untermodul, Satz)` - leerer Name heisst das Wurzelmodul.
///
/// Zeilenzahl: siehe unten; die Tabelle ist erzeugt.
pub const MODULDOKU: &[(&str, &str)] = &[
    (r#""#, r#"Pygame is a set of Python modules designed for writing games.
It is written on top of the excellent SDL library. This allows you
to create fully featured games and multimedia programs in the python
language. The package is highly portable, with games running on
Windows, macOS, OS X, BeOS, FreeBSD, IRIX, and Linux."#),
    (r#"_camera"#, r#"pygame module for camera use"#),
    (r#"_debug"#, r#"Debug functionality that allows for more useful issue reporting"#),
    (r#"_freetype"#, r#"pygame module for loading and rendering computer fonts"#),
    (r#"_render"#, r#"docs_needed"#),
    (r#"bufferproxy"#, r#"BufferProxy(parent) -> BufferProxy
pygame object to export a surface buffer through an array protocol"#),
    (r#"color"#, r#"color module for pygame"#),
    (r#"colordict"#, r#"A dictionary of RGBA tuples indexed by color names.

See https://pyga.me/docs/ref/color_list.html for sample swatches.
"#),
    (r#"constants"#, r#"Constants defined by SDL and needed in Pygame.
"#),
    (r#"cursors"#, r#"Pygame module for cursor resources.

Pygame offers control over the system hardware cursor. Pygame supports
black and white cursors (bitmap cursors), as well as system variant cursors and color cursors.
You control the cursor with functions inside :mod:`pygame.mouse`.

This cursors module contains functions for loading and decoding various
cursor formats. These allow you to easily store your cursors in external files
or directly as encoded python strings.

The module includes several standard cursors. The :func:`pygame.mouse.set_cursor()`
function takes several arguments. All those arguments have been stored in a
single tuple you can call like this:

::

   >>> pygame.mouse.set_cursor(*pygame.cursors.arrow)

The following variables can be passed to ``pygame.mouse.set_cursor`` function:

   * ``pygame.cursors.arrow``

   * ``pygame.cursors.diamond``

   * ``pygame.cursors.broken_x``

   * ``pygame.cursors.tri_left``

   * ``pygame.cursors.tri_right``

This module also contains a few cursors as formatted strings. You'll need to
pass these to ``pygame.cursors.compile()`` function before you can use them.
The example call would look like this:

::

   >>> cursor = pygame.cursors.compile(pygame.cursors.textmarker_strings)
   >>> pygame.mouse.set_cursor((8, 16), (0, 0), *cursor)

The following strings can be converted into cursor bitmaps with
``pygame.cursors.compile()`` :

   * ``pygame.cursors.thickarrow_strings``

   * ``pygame.cursors.sizer_x_strings``

   * ``pygame.cursors.sizer_y_strings``

   * ``pygame.cursors.sizer_xy_strings``

   * ``pygame.cursor.textmarker_strings``
"#),
    (r#"display"#, r#"Pygame module to control the display window and screen."#),
    (r#"draw"#, r#"Pygame module for drawing shapes."#),
    (r#"event"#, r#"pygame module for interacting with events and queues"#),
    (r#"font"#, r#"pygame module for loading and rendering fonts"#),
    (r#"freetype"#, r#"Enhanced Pygame module for loading and rendering computer fonts"#),
    (r#"ftfont"#, r#"pygame module for loading and rendering fonts (freetype alternative)"#),
    (r#"geometry"#, r#"Module for the Line, Circle and Polygon objects
"#),
    (r#"gfxdraw"#, r#"pygame module for drawing shapes"#),
    (r#"image"#, r#"Pygame module for image transfer."#),
    (r#"imageext"#, r#"additional image loaders"#),
    (r#"joystick"#, r#"Joystick(id, /) -> Joystick
Create a new Joystick object."#),
    (r#"key"#, r#"Pygame module to work with the keyboard."#),
    (r#"locals"#, r#"Set of functions from pygame that are handy to have in
the local namespace for your module"#),
    (r#"mask"#, r#"pygame module for image masks."#),
    (r#"math"#, r#"pygame module for vector classes"#),
    (r#"midi"#, r#"pygame.midi
pygame module for interacting with midi input and output.

The midi module can send output to midi devices, and get input
from midi devices.  It can also list midi devices on the system.

Including real midi devices, and virtual ones.

It uses the portmidi library.  Is portable to which ever platforms
portmidi supports (currently windows, OSX, and linux).

This uses pyportmidi for now, but may use its own bindings at some
point in the future.  The pyportmidi bindings are included with pygame.

New in pygame 1.9.0.
"#),
    (r#"mixer"#, r#"pygame module for loading and playing sounds"#),
    (r#"mixer_music"#, r#"pygame module for controlling streamed audio"#),
    (r#"mouse"#, r#"Pygame module to work with the mouse."#),
    (r#"newbuffer"#, r#"exports BufferMixin, add a new buffer interface to a class"#),
    (r#"pixelarray"#, r#"PixelArray(Surface) -> PixelArray
pygame object for direct pixel access of surfaces"#),
    (r#"pixelcopy"#, r#"pygame module for general pixel array copying"#),
    (r#"pkgdata"#, r#"
pkgdata is a simple, extensible way for a package to acquire data file
resources.

The getResource function is equivalent to the standard idioms, such as
the following minimal implementation:

    import sys, os

    def getResource(identifier, pkgname=__name__):
        pkgpath = os.path.dirname(sys.modules[pkgname].__file__)
        path = os.path.join(pkgpath, identifier)
        return file(os.path.normpath(path), mode='rb')

When a __loader__ is present on the module given by __name__, it will defer
getResource to its get_data implementation and return it as a file-like
object (such as StringIO).
"#),
    (r#"rect"#, r#"Module for the rectangle object
"#),
    (r#"rwobject"#, r#"SDL_RWops support"#),
    (r#"scrap"#, r#"pygame module for clipboard support."#),
    (r#"sndarray"#, r#"pygame module for accessing sound sample data

Functions to convert between NumPy arrays and Sound objects. This module
will only be functional when pygame can use the external NumPy package.
If NumPy can't be imported, surfarray becomes a MissingModule object.

Sound data is made of thousands of samples per second, and each sample
is the amplitude of the wave at a particular moment in time. For
example, in 22-kHz format, element number 5 of the array is the
amplitude of the wave after 5/22000 seconds.

Each sample is an 8-bit or 16-bit integer, depending on the data format.
A stereo sound file has two values per sample, while a mono sound file
only has one.

Sounds with 16-bit data will be treated as unsigned integers,
if the sound sample type requests this.
"#),
    (r#"sprite"#, r#"pygame module with basic game object classes

This module contains several simple classes to be used within games. There
are the main Sprite class and several Group classes that contain Sprites.
The use of these classes is entirely optional when using Pygame. The classes
are fairly lightweight and only provide a starting place for the code
that is common to most games.

The Sprite class is intended to be used as a base class for the different
types of objects in the game. There is also a base Group class that simply
stores sprites. A game could create new types of Group classes that operate
on specially customized Sprite instances they contain.

The basic Sprite class can draw the Sprites it contains to a Surface. The
Group.draw() method requires that each Sprite have a Surface.image attribute
and a Surface.rect. The Group.clear() method requires these same attributes
and can be used to erase all the Sprites with background. There are also
more advanced Groups: pygame.sprite.RenderUpdates().

Lastly, this module contains several collision functions. These help find
sprites inside multiple groups that have intersecting bounding rectangles.
To find the collisions, the Sprites are required to have a Surface.rect
attribute assigned.

The groups are designed for high efficiency in removing and adding Sprites
to them. They also allow cheap testing to see if a Sprite already exists in
a Group. A given Sprite can exist in any number of groups. A game could use
some groups to control object rendering, and a completely separate set of
groups to control interaction or player movement. Instead of adding type
attributes or bools to a derived Sprite class, consider keeping the
Sprites inside organized Groups. This will allow for easier lookup later
in the game.

Sprites and Groups manage their relationships with the add() and remove()
methods. These methods can accept a single or multiple group arguments for
membership.  The default initializers for these classes also take a
single group or list of groups as arguments for initial membership. It is safe
to repeatedly add and remove the same Sprite from a Group.

While it is possible to design sprite and group classes that don't derive
from the Sprite and AbstractGroup classes below, it is strongly recommended
that you extend those when you create a new Sprite or Group class.

Sprites are not thread safe, so lock them yourself if using threads.

"#),
    (r#"surface"#, r#"Surface(size, flags=0, depth=0, masks=None) -> Surface
Surface(size, flags=0, surface=...) -> Surface
Pygame object for representing images."#),
    (r#"surfarray"#, r#"pygame module for accessing surface pixel data using array interfaces

Functions to convert between NumPy arrays and Surface objects. This module
will only be functional when pygame can use the external NumPy package.
If NumPy can't be imported, surfarray becomes a MissingModule object.

Every pixel is stored as a single integer value to represent the red,
green, and blue colors. The 8bit images use a value that looks into a
colormap. Pixels with higher depth use a bit packing process to place
three or four values into a single number.

The arrays are indexed by the X axis first, followed by the Y
axis. Arrays that treat the pixels as a single integer are referred to
as 2D arrays. This module can also separate the red, green, and blue
color values into separate indices. These types of arrays are referred
to as 3D arrays, and the last index is 0 for red, 1 for green, and 2 for
blue.
"#),
    (r#"surflock"#, r#"Surface locking support"#),
    (r#"sysfont"#, r#"sysfont, used in the font module to find system fonts"#),
    (r#"system"#, r#"Pygame module to provide additional context about the system."#),
    (r#"time"#, r#"Pygame module for monitoring time."#),
    (r#"transform"#, r#"Pygame module to transform surfaces."#),
    (r#"typing"#, r#"Set of common pygame type aliases for proper typehint annotations"#),
    (r#"version"#, r#"Simply the current installed pygame version. The version information is
stored in the regular pygame module as 'pygame.ver'. Keeping the version
information also available in a separate module allows you to test the
pygame version without importing the main pygame module.

The python version information should always compare greater than any previous
releases. (hmm, until we get to versions > 10)
"#),
    (r#"window"#, r#"docs_needed"#),
];

/// `(Untermodul, Klasse, Satz)` - leerer Modulname heisst das Wurzelmodul.
///
/// Dieselbe Sache fuer die Klassen: `help(pygame.Surface)` zeigt
/// `Surface(size, flags=0, depth=0, masks=None) -> Surface` und darunter
/// den Satz. Eine Klasse kann unter mehreren Namen stehen
/// (`Channel`, `mixer.Channel`, `mixer.ChannelType` sind **ein** Objekt);
/// die Tabelle fuehrt jeden Weg, und derselbe Satz zweimal zu setzen
/// kostet nichts.
pub const KLASSENDOKU: &[(&str, &str, &str)] = &[
    (r#""#, r#"BufferProxy"#, r#"BufferProxy(parent) -> BufferProxy
pygame object to export a surface buffer through an array protocol"#),
    (r#""#, r#"Channel"#, r#"Channel(id) -> Channel
Create a Channel object for controlling playback"#),
    (r#""#, r#"Clock"#, r#"Clock() -> Clock
Create an object to help track time."#),
    (r#""#, r#"Color"#, r#"Color(r, g, b, a=255) -> Color
Color(rgbvalue) -> Color
Pygame object for color representations."#),
    (r#""#, r#"Cursor"#, r#"Pygame object representing a cursor.

In pygame 2, there are 3 types of cursors you can create to give your
game that little bit of extra polish. There's **bitmap** type cursors,
which existed in pygame 1.x, and are compiled from a string or load from an xbm file.
Then there are **system** type cursors, where you choose a preset that will
convey the same meaning but look native across different operating systems.
Finally you can create a **color** cursor, which displays a pygame surface as the cursor.

**Creating a system cursor**

Choose a constant from this list, pass it into ``pygame.cursors.Cursor(constant)``,
and you're good to go. Be advised that not all systems support every system
cursor, and you may get a substitution instead. For example, on macOS,
WAIT/WAITARROW should show up as an arrow, and SIZENWSE/SIZENESW/SIZEALL
should show up as a closed hand. And on Wayland, every SIZE cursor should
show up as a hand.

::

   Pygame Cursor Constant           Description
   --------------------------------------------
   pygame.SYSTEM_CURSOR_ARROW       arrow
   pygame.SYSTEM_CURSOR_IBEAM       i-beam
   pygame.SYSTEM_CURSOR_WAIT        wait
   pygame.SYSTEM_CURSOR_CROSSHAIR   crosshair
   pygame.SYSTEM_CURSOR_WAITARROW   small wait cursor
                                    (or wait if not available)
   pygame.SYSTEM_CURSOR_SIZENWSE    double arrow pointing
                                    northwest and southeast
   pygame.SYSTEM_CURSOR_SIZENESW    double arrow pointing
                                    northeast and southwest
   pygame.SYSTEM_CURSOR_SIZEWE      double arrow pointing
                                    west and east
   pygame.SYSTEM_CURSOR_SIZENS      double arrow pointing
                                    north and south
   pygame.SYSTEM_CURSOR_SIZEALL     four pointed arrow pointing
                                    north, south, east, and west
   pygame.SYSTEM_CURSOR_NO          slashed circle or crossbones
   pygame.SYSTEM_CURSOR_HAND        hand

**Creating a cursor without passing arguments**

In addition to the cursor constants available and described above, you can
also call ``pygame.cursors.Cursor()``, and your cursor is ready (doing that
is the same as calling ``pygame.cursors.Cursor(pygame.SYSTEM_CURSOR_ARROW)``.
Doing one of those calls actually creates a system cursor using the default
native image.

**Creating a color cursor**

To create a color cursor, create a ``Cursor`` from a ``hotspot`` and a ``surface``.
``hotspot`` are (x,y) coordinates that determine where in the cursor the exact point is.
The hotspot position must be within the bounds of the ``surface``.

**Creating a bitmap cursor**

When the mouse cursor is visible, it will be displayed as a black and white
bitmap using the given bitmask arrays. The ``size`` is a sequence containing
the cursor width and height. ``hotspot`` is a sequence containing the cursor
hotspot position.

A cursor has a width and height, but a mouse position is represented by a
set of point coordinates. So the value passed into the cursor ``hotspot``
variable helps pygame to actually determine at what exact point the cursor
is at.

``xormasks`` is a sequence of bytes containing the cursor xor data masks.
Lastly ``andmasks``, a sequence of bytes containing the cursor bitmask data.
To create these variables, we can make use of the
:func:`pygame.cursors.compile()` function.

Width and height must be a multiple of 8, and the mask arrays must be the
correct size for the given width and height. Otherwise an exception is raised.

.. versionaddedold:: 2.0.1
"#),
    (r#""#, r#"Event"#, r#"Event(type, dict) -> Event
Event(type, **attributes) -> Event
pygame object for representing events"#),
    (r#""#, r#"FRect"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#""#, r#"Font"#, r#"Font(filename=None, size=20) -> Font
Font(filename, size) -> Font
Font(pathlib.Path, size) -> Font
Font(object, size) -> Font
create a new Font object from a file"#),
    (r#""#, r#"Mask"#, r#"Mask(size=(width, height)) -> Mask
Mask(size=(width, height), fill=False) -> Mask
pygame object for representing 2D bitmasks"#),
    (r#""#, r#"PixelArray"#, r#"PixelArray(Surface) -> PixelArray
pygame object for direct pixel access of surfaces"#),
    (r#""#, r#"Rect"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#""#, r#"Sound"#, r#"Sound(filename) -> Sound
Sound(file=filename) -> Sound
Sound(file=pathlib_path) -> Sound
Sound(buffer) -> Sound
Sound(buffer=buffer) -> Sound
Sound(object) -> Sound
Sound(file=object) -> Sound
Sound(array=object) -> Sound
Create a new Sound object from a file or buffer object"#),
    (r#""#, r#"Surface"#, r#"Surface(size, flags=0, depth=0, masks=None) -> Surface
Surface(size, flags=0, surface=...) -> Surface
Pygame object for representing images."#),
    (r#""#, r#"SurfaceType"#, r#"Surface(size, flags=0, depth=0, masks=None) -> Surface
Surface(size, flags=0, surface=...) -> Surface
Pygame object for representing images."#),
    (r#""#, r#"Vector2"#, r#"Vector2() -> Vector2(0, 0)
Vector2(int) -> Vector2
Vector2(float) -> Vector2
Vector2(Vector2) -> Vector2
Vector2(x, y) -> Vector2
Vector2((x, y)) -> Vector2
a 2-Dimensional Vector"#),
    (r#""#, r#"Vector3"#, r#"Vector3() -> Vector3(0, 0, 0)
Vector3(int) -> Vector3
Vector3(float) -> Vector3
Vector3(Vector3) -> Vector3
Vector3(x, y, z) -> Vector3
Vector3((x, y, z)) -> Vector3
a 3-Dimensional Vector"#),
    (r#""#, r#"Window"#, r#"Window(title='pygame window', size=(640, 480), position=WINDOWPOS_UNDEFINED, *, fullscreen=..., fullscreen_desktop=..., opengl=..., vulkan=..., hidden=..., borderless=..., resizable=..., minimized=..., maximized=..., mouse_grabbed=..., keyboard_grabbed=..., input_focus=..., mouse_focus=..., allow_high_dpi=..., mouse_capture=..., always_on_top=..., utility=...) -> Window
Pygame object that represents a window."#),
    (r#"_camera"#, r#"Camera"#, r#"Camera(device, (width, height), format) -> Camera
load a camera"#),
    (r#"_data_classes"#, r#"PowerState"#, r#"PowerState(battery_percent: int | None, battery_seconds: int | None, on_battery: bool, no_battery: bool, charging: bool, charged: bool, plugged_in: bool, has_battery: bool)"#),
    (r#"_freetype"#, r#"Font"#, r#"Font(file, size=0, font_index=0, resolution=0, ucs4=False) -> Font
Font(pathlib.Path) -> Font
Create a new Font instance from a supported font file."#),
    (r#"_render"#, r#"Image"#, r#"Image(texture_or_image, srcrect=None) -> Image
pygame object that represents a portion of a texture"#),
    (r#"_render"#, r#"Renderer"#, r#"Renderer(window, index=-1, accelerated=-1, vsync=False, target_texture=False) -> Renderer
pygame object wrapping a 2D rendering context for a window"#),
    (r#"_render"#, r#"Texture"#, r#"Texture(renderer, size, depth=0, static=False, streaming=False, target=False, scale_quality=None) -> Texture
pygame object that represents a texture"#),
    (r#"_sdl2"#, r#"Window"#, r#"Window(title='pygame window', size=(640, 480), position=WINDOWPOS_UNDEFINED, *, fullscreen=..., fullscreen_desktop=..., opengl=..., vulkan=..., hidden=..., borderless=..., resizable=..., minimized=..., maximized=..., mouse_grabbed=..., keyboard_grabbed=..., input_focus=..., mouse_focus=..., allow_high_dpi=..., mouse_capture=..., always_on_top=..., utility=...) -> Window
Pygame object that represents a window."#),
    (r#"_sprite"#, r#"AbstractGroup"#, r#"base class for containers of sprites

AbstractGroup does everything needed to behave as a normal group. You can
easily subclass a new group class from this or the other groups below if
you want to add more features.

Any AbstractGroup-derived sprite groups act like sequences and support
iteration, len, and so on.

"#),
    (r#"_sprite"#, r#"DirtySprite"#, r#"a more featureful subclass of Sprite with more attributes

pygame.sprite.DirtySprite(*groups): return DirtySprite

Extra DirtySprite attributes with their default values:

dirty = 1
    If set to 1, it is repainted and then set to 0 again.
    If set to 2, it is always dirty (repainted each frame;
    flag is not reset).
    If set to 0, it is not dirty and therefore not repainted again.

blendmode = 0
    It's the special_flags argument of Surface.blit; see the blendmodes in
    the Surface.blit documentation

source_rect = None
    This is the source rect to use. Remember that it is relative to the top
    left corner (0, 0) of self.image.

visible = 1
    Normally this is 1. If set to 0, it will not be repainted. (If you
    change visible to 1, you must set dirty to 1 for it to be erased from
    the screen.)

_layer = 0
    0 is the default value but this can be set differently
    when subclassing.

"#),
    (r#"_sprite"#, r#"Group"#, r#"container class for many Sprites

pygame.sprite.Group(*sprites): return Group

A simple container for Sprite objects. This class can be subclassed to
create containers with more specific behaviors. The constructor takes any
number of Sprite arguments to add to the Group. The group supports the
following standard Python operations:

    in      test if a Sprite is contained
    len     the number of Sprites contained
    bool    test if any Sprites are contained
    iter    iterate through all the Sprites

The Sprites in the Group are not ordered, so the Sprites are drawn and
iterated over in no particular order.

"#),
    (r#"_sprite"#, r#"GroupSingle"#, r#"A group container that holds a single most recent item.

This class works just like a regular group, but it only keeps a single
sprite in the group. Whatever sprite has been added to the group last will
be the only sprite in the group.

You can access its one sprite as the .sprite attribute.  Assigning to this
attribute will properly remove the old sprite and then add the new one.

"#),
    (r#"_sprite"#, r#"LayeredDirty"#, r#"LayeredDirty Group is for DirtySprites; subclasses LayeredUpdates

pygame.sprite.LayeredDirty(*sprites, **kwargs): return LayeredDirty

This group requires pygame.sprite.DirtySprite or any sprite that
has the following attributes:
    image, rect, dirty, visible, blendmode (see doc of DirtySprite).

It uses the dirty flag technique and is therefore faster than
pygame.sprite.RenderUpdates if you have many static sprites.  It
also switches automatically between dirty rect updating and full
screen drawing, so you do not have to worry which would be faster.

As with the pygame.sprite.Group, you can specify some additional attributes
through kwargs:
    _use_update: True/False   (default is False)
    _default_layer: default layer where the sprites without a layer are
        added
    _time_threshold: threshold time for switching between dirty rect mode
        and fullscreen mode; defaults to updating at 80 frames per second,
        which is equal to 1000.0 / 80.0

New in pygame 1.8.0

"#),
    (r#"_sprite"#, r#"LayeredUpdates"#, r#"LayeredUpdates Group handles layers, which are drawn like RenderUpdates

pygame.sprite.LayeredUpdates(*sprites, **kwargs): return LayeredUpdates

This group is fully compatible with pygame.sprite.Sprite.
New in pygame 1.8.0

"#),
    (r#"_sprite"#, r#"RenderUpdates"#, r#"Group class that tracks dirty updates

pygame.sprite.RenderUpdates(*sprites): return RenderUpdates

This class is derived from pygame.sprite.Group(). It has an enhanced draw
method that tracks the changed areas of the screen.

"#),
    (r#"_sprite"#, r#"Sprite"#, r#"simple base class for visible game objects

pygame.sprite.Sprite(*groups): return Sprite

The base class for visible game objects. Derived classes will want to
override the Sprite.update() method and assign Sprite.image and Sprite.rect
attributes.  The initializer can accept any number of Group instances that
the Sprite will become a member of.

When subclassing the Sprite class, be sure to call the base initializer
before adding the Sprite to Groups.

"#),
    (r#"_sprite"#, r#"collide_circle_ratio"#, r#"detect collision between two sprites using scaled circles

This callable class checks for collisions between two sprites using a
scaled version of a sprite's radius. It is created with a ratio as the
argument to the constructor. The instance is then intended to be passed as
a collided callback function to the *collide functions.

New in pygame 1.8.1

"#),
    (r#"_sprite"#, r#"collide_rect_ratio"#, r#"A callable class that checks for collisions using scaled rects

The class checks for collisions between two sprites using a scaled version
of the sprites' rects. Is created with a ratio; the instance is then
intended to be passed as a collided callback function to the *collide
functions.

New in pygame 1.8.1

"#),
    (r#"bufferproxy"#, r#"BufferProxy"#, r#"BufferProxy(parent) -> BufferProxy
pygame object to export a surface buffer through an array protocol"#),
    (r#"camera"#, r#"ABC"#, r#"Helper class that provides a standard way to create an ABC using
inheritance.
"#),
    (r#"color"#, r#"Color"#, r#"Color(r, g, b, a=255) -> Color
Color(rgbvalue) -> Color
Pygame object for color representations."#),
    (r#"cursors"#, r#"Cursor"#, r#"Pygame object representing a cursor.

In pygame 2, there are 3 types of cursors you can create to give your
game that little bit of extra polish. There's **bitmap** type cursors,
which existed in pygame 1.x, and are compiled from a string or load from an xbm file.
Then there are **system** type cursors, where you choose a preset that will
convey the same meaning but look native across different operating systems.
Finally you can create a **color** cursor, which displays a pygame surface as the cursor.

**Creating a system cursor**

Choose a constant from this list, pass it into ``pygame.cursors.Cursor(constant)``,
and you're good to go. Be advised that not all systems support every system
cursor, and you may get a substitution instead. For example, on macOS,
WAIT/WAITARROW should show up as an arrow, and SIZENWSE/SIZENESW/SIZEALL
should show up as a closed hand. And on Wayland, every SIZE cursor should
show up as a hand.

::

   Pygame Cursor Constant           Description
   --------------------------------------------
   pygame.SYSTEM_CURSOR_ARROW       arrow
   pygame.SYSTEM_CURSOR_IBEAM       i-beam
   pygame.SYSTEM_CURSOR_WAIT        wait
   pygame.SYSTEM_CURSOR_CROSSHAIR   crosshair
   pygame.SYSTEM_CURSOR_WAITARROW   small wait cursor
                                    (or wait if not available)
   pygame.SYSTEM_CURSOR_SIZENWSE    double arrow pointing
                                    northwest and southeast
   pygame.SYSTEM_CURSOR_SIZENESW    double arrow pointing
                                    northeast and southwest
   pygame.SYSTEM_CURSOR_SIZEWE      double arrow pointing
                                    west and east
   pygame.SYSTEM_CURSOR_SIZENS      double arrow pointing
                                    north and south
   pygame.SYSTEM_CURSOR_SIZEALL     four pointed arrow pointing
                                    north, south, east, and west
   pygame.SYSTEM_CURSOR_NO          slashed circle or crossbones
   pygame.SYSTEM_CURSOR_HAND        hand

**Creating a cursor without passing arguments**

In addition to the cursor constants available and described above, you can
also call ``pygame.cursors.Cursor()``, and your cursor is ready (doing that
is the same as calling ``pygame.cursors.Cursor(pygame.SYSTEM_CURSOR_ARROW)``.
Doing one of those calls actually creates a system cursor using the default
native image.

**Creating a color cursor**

To create a color cursor, create a ``Cursor`` from a ``hotspot`` and a ``surface``.
``hotspot`` are (x,y) coordinates that determine where in the cursor the exact point is.
The hotspot position must be within the bounds of the ``surface``.

**Creating a bitmap cursor**

When the mouse cursor is visible, it will be displayed as a black and white
bitmap using the given bitmask arrays. The ``size`` is a sequence containing
the cursor width and height. ``hotspot`` is a sequence containing the cursor
hotspot position.

A cursor has a width and height, but a mouse position is represented by a
set of point coordinates. So the value passed into the cursor ``hotspot``
variable helps pygame to actually determine at what exact point the cursor
is at.

``xormasks`` is a sequence of bytes containing the cursor xor data masks.
Lastly ``andmasks``, a sequence of bytes containing the cursor bitmask data.
To create these variables, we can make use of the
:func:`pygame.cursors.compile()` function.

Width and height must be a multiple of 8, and the mask arrays must be the
correct size for the given width and height. Otherwise an exception is raised.

.. versionaddedold:: 2.0.1
"#),
    (r#"docs"#, r#"Path"#, r#"PurePath subclass that can make system calls.

Path represents a filesystem path but unlike PurePath, also offers
methods to do system calls on path objects. Depending on your system,
instantiating a Path will return either a PosixPath or a WindowsPath
object. You can also instantiate a PosixPath or WindowsPath directly,
but cannot instantiate a WindowsPath on a POSIX system or vice versa.
"#),
    (r#"event"#, r#"Event"#, r#"Event(type, dict) -> Event
Event(type, **attributes) -> Event
pygame object for representing events"#),
    (r#"event"#, r#"EventType"#, r#"Event(type, dict) -> Event
Event(type, **attributes) -> Event
pygame object for representing events"#),
    (r#"font"#, r#"Font"#, r#"Font(filename=None, size=20) -> Font
Font(filename, size) -> Font
Font(pathlib.Path, size) -> Font
Font(object, size) -> Font
create a new Font object from a file"#),
    (r#"font"#, r#"FontType"#, r#"Font(filename=None, size=20) -> Font
Font(filename, size) -> Font
Font(pathlib.Path, size) -> Font
Font(object, size) -> Font
create a new Font object from a file"#),
    (r#"freetype"#, r#"Font"#, r#"Font(file, size=0, font_index=0, resolution=0, ucs4=False) -> Font
Font(pathlib.Path) -> Font
Create a new Font instance from a supported font file."#),
    (r#"ftfont"#, r#"Font"#, r#"Font(filename, size) -> Font
Font(object, size) -> Font
create a new Font object from a file (freetype alternative)

This Font type differs from font.Font in that it can render glyphs
for Unicode code points in the supplementary planes (> 0xFFFF).
"#),
    (r#"ftfont"#, r#"FontType"#, r#"Font(filename, size) -> Font
Font(object, size) -> Font
create a new Font object from a file (freetype alternative)

This Font type differs from font.Font in that it can render glyphs
for Unicode code points in the supplementary planes (> 0xFFFF).
"#),
    (r#"geometry"#, r#"Circle"#, r#"Circle((x, y), radius) -> Circle
Circle(x, y, radius) -> Circle
pygame object for representing a circle"#),
    (r#"geometry"#, r#"Line"#, r#"Line((ax, ay), (bx, by)) -> Line
Line(ax, ay, bx, by) -> Line
pygame object for representing a line"#),
    (r#"joystick"#, r#"JoystickType"#, r#"Joystick(id, /) -> Joystick
Create a new Joystick object."#),
    (r#"locals"#, r#"Color"#, r#"Color(r, g, b, a=255) -> Color
Color(rgbvalue) -> Color
Pygame object for color representations."#),
    (r#"locals"#, r#"Rect"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#"mask"#, r#"Mask"#, r#"Mask(size=(width, height)) -> Mask
Mask(size=(width, height), fill=False) -> Mask
pygame object for representing 2D bitmasks"#),
    (r#"mask"#, r#"MaskType"#, r#"Mask(size=(width, height)) -> Mask
Mask(size=(width, height), fill=False) -> Mask
pygame object for representing 2D bitmasks"#),
    (r#"math"#, r#"Vector2"#, r#"Vector2() -> Vector2(0, 0)
Vector2(int) -> Vector2
Vector2(float) -> Vector2
Vector2(Vector2) -> Vector2
Vector2(x, y) -> Vector2
Vector2((x, y)) -> Vector2
a 2-Dimensional Vector"#),
    (r#"math"#, r#"Vector3"#, r#"Vector3() -> Vector3(0, 0, 0)
Vector3(int) -> Vector3
Vector3(float) -> Vector3
Vector3(Vector3) -> Vector3
Vector3(x, y, z) -> Vector3
Vector3((x, y, z)) -> Vector3
a 3-Dimensional Vector"#),
    (r#"midi"#, r#"Input"#, r#"Input is used to get midi input from midi devices.
Input(device_id)
Input(device_id, buffer_size)

buffer_size - the number of input events to be buffered waiting to
  be read using Input.read()
"#),
    (r#"midi"#, r#"MidiException"#, r#"exception that pygame.midi functions and classes can raise
MidiException(errno)
"#),
    (r#"midi"#, r#"Output"#, r#"Output is used to send midi to an output device
Output(device_id)
Output(device_id, latency = 0)
Output(device_id, buffer_size = 4096)
Output(device_id, latency, buffer_size)

The buffer_size specifies the number of output events to be
buffered waiting for output.  (In some cases -- see below --
PortMidi does not buffer output at all and merely passes data
to a lower-level API, in which case buffersize is ignored.)

latency is the delay in milliseconds applied to timestamps to determine
when the output should actually occur. (If latency is < 0, 0 is
assumed.)

If latency is zero, timestamps are ignored and all output is delivered
immediately. If latency is greater than zero, output is delayed until
the message timestamp plus the latency. (NOTE: time is measured
relative to the time source indicated by time_proc. Timestamps are
absolute, not relative delays or offsets.) In some cases, PortMidi
can obtain better timing than your application by passing timestamps
along to the device driver or hardware. Latency may also help you
to synchronize midi data to audio data by matching midi latency to
the audio buffer latency.

"#),
    (r#"mixer"#, r#"Channel"#, r#"Channel(id) -> Channel
Create a Channel object for controlling playback"#),
    (r#"mixer"#, r#"ChannelType"#, r#"Channel(id) -> Channel
Create a Channel object for controlling playback"#),
    (r#"mixer"#, r#"Sound"#, r#"Sound(filename) -> Sound
Sound(file=filename) -> Sound
Sound(file=pathlib_path) -> Sound
Sound(buffer) -> Sound
Sound(buffer=buffer) -> Sound
Sound(object) -> Sound
Sound(file=object) -> Sound
Sound(array=object) -> Sound
Create a new Sound object from a file or buffer object"#),
    (r#"mixer"#, r#"SoundType"#, r#"Sound(filename) -> Sound
Sound(file=filename) -> Sound
Sound(file=pathlib_path) -> Sound
Sound(buffer) -> Sound
Sound(buffer=buffer) -> Sound
Sound(object) -> Sound
Sound(file=object) -> Sound
Sound(array=object) -> Sound
Create a new Sound object from a file or buffer object"#),
    (r#"newbuffer"#, r#"BufferMixin"#, r#"Python level new buffer protocol exporter
"#),
    (r#"newbuffer"#, r#"Py_buffer"#, r#"Python level Py_buffer struct wrapper
"#),
    (r#"pixelarray"#, r#"PixelArray"#, r#"PixelArray(Surface) -> PixelArray
pygame object for direct pixel access of surfaces"#),
    (r#"pypm"#, r#"Input"#, r#"Represents an input MIDI stream device.

    Takes the form::

        input = pypm.Input(input_device)

    "#),
    (r#"pypm"#, r#"Output"#, r#"Represents an output MIDI stream device.

    Takes the form::

        output = pypm.Output(output_device, latency)

    latency is in ms. If latency == 0 then timestamps for output are ignored.

    "#),
    (r#"rect"#, r#"FRect"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#"rect"#, r#"FRectType"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#"rect"#, r#"Rect"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#"rect"#, r#"RectType"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#"sprite"#, r#"AbstractGroup"#, r#"base class for containers of sprites

AbstractGroup does everything needed to behave as a normal group. You can
easily subclass a new group class from this or the other groups below if
you want to add more features.

Any AbstractGroup-derived sprite groups act like sequences and support
iteration, len, and so on.

"#),
    (r#"sprite"#, r#"DirtySprite"#, r#"a more featureful subclass of Sprite with more attributes

pygame.sprite.DirtySprite(*groups): return DirtySprite

Extra DirtySprite attributes with their default values:

dirty = 1
    If set to 1, it is repainted and then set to 0 again.
    If set to 2, it is always dirty (repainted each frame;
    flag is not reset).
    If set to 0, it is not dirty and therefore not repainted again.

blendmode = 0
    It's the special_flags argument of Surface.blit; see the blendmodes in
    the Surface.blit documentation

source_rect = None
    This is the source rect to use. Remember that it is relative to the top
    left corner (0, 0) of self.image.

visible = 1
    Normally this is 1. If set to 0, it will not be repainted. (If you
    change visible to 1, you must set dirty to 1 for it to be erased from
    the screen.)

_layer = 0
    0 is the default value but this can be set differently
    when subclassing.

"#),
    (r#"sprite"#, r#"Group"#, r#"container class for many Sprites

pygame.sprite.Group(*sprites): return Group

A simple container for Sprite objects. This class can be subclassed to
create containers with more specific behaviors. The constructor takes any
number of Sprite arguments to add to the Group. The group supports the
following standard Python operations:

    in      test if a Sprite is contained
    len     the number of Sprites contained
    bool    test if any Sprites are contained
    iter    iterate through all the Sprites

The Sprites in the Group are not ordered, so the Sprites are drawn and
iterated over in no particular order.

"#),
    (r#"sprite"#, r#"GroupSingle"#, r#"A group container that holds a single most recent item.

This class works just like a regular group, but it only keeps a single
sprite in the group. Whatever sprite has been added to the group last will
be the only sprite in the group.

You can access its one sprite as the .sprite attribute.  Assigning to this
attribute will properly remove the old sprite and then add the new one.

"#),
    (r#"sprite"#, r#"LayeredDirty"#, r#"LayeredDirty Group is for DirtySprites; subclasses LayeredUpdates

pygame.sprite.LayeredDirty(*sprites, **kwargs): return LayeredDirty

This group requires pygame.sprite.DirtySprite or any sprite that
has the following attributes:
    image, rect, dirty, visible, blendmode (see doc of DirtySprite).

It uses the dirty flag technique and is therefore faster than
pygame.sprite.RenderUpdates if you have many static sprites.  It
also switches automatically between dirty rect updating and full
screen drawing, so you do not have to worry which would be faster.

As with the pygame.sprite.Group, you can specify some additional attributes
through kwargs:
    _use_update: True/False   (default is False)
    _default_layer: default layer where the sprites without a layer are
        added
    _time_threshold: threshold time for switching between dirty rect mode
        and fullscreen mode; defaults to updating at 80 frames per second,
        which is equal to 1000.0 / 80.0

New in pygame 1.8.0

"#),
    (r#"sprite"#, r#"LayeredUpdates"#, r#"LayeredUpdates Group handles layers, which are drawn like RenderUpdates

pygame.sprite.LayeredUpdates(*sprites, **kwargs): return LayeredUpdates

This group is fully compatible with pygame.sprite.Sprite.
New in pygame 1.8.0

"#),
    (r#"sprite"#, r#"Rect"#, r#"Rect(left, top, width, height) -> Rect
Rect((left, top), (width, height)) -> Rect
Rect(object) -> Rect
Rect() -> Rect
FRect(left, top, width, height) -> FRect
FRect((left, top), (width, height)) -> FRect
FRect(object) -> FRect
FRect() -> FRect
pygame object for storing rectangular coordinates"#),
    (r#"sprite"#, r#"RenderUpdates"#, r#"Group class that tracks dirty updates

pygame.sprite.RenderUpdates(*sprites): return RenderUpdates

This class is derived from pygame.sprite.Group(). It has an enhanced draw
method that tracks the changed areas of the screen.

"#),
    (r#"sprite"#, r#"Sprite"#, r#"simple base class for visible game objects

pygame.sprite.Sprite(*groups): return Sprite

The base class for visible game objects. Derived classes will want to
override the Sprite.update() method and assign Sprite.image and Sprite.rect
attributes.  The initializer can accept any number of Group instances that
the Sprite will become a member of.

When subclassing the Sprite class, be sure to call the base initializer
before adding the Sprite to Groups.

"#),
    (r#"sprite"#, r#"collide_circle_ratio"#, r#"detect collision between two sprites using scaled circles

This callable class checks for collisions between two sprites using a
scaled version of a sprite's radius. It is created with a ratio as the
argument to the constructor. The instance is then intended to be passed as
a collided callback function to the *collide functions.

New in pygame 1.8.1

"#),
    (r#"sprite"#, r#"collide_rect_ratio"#, r#"A callable class that checks for collisions using scaled rects

The class checks for collisions between two sprites using a scaled version
of the sprites' rects. Is created with a ratio; the instance is then
intended to be passed as a collided callback function to the *collide
functions.

New in pygame 1.8.1

"#),
    (r#"surface"#, r#"Surface"#, r#"Surface(size, flags=0, depth=0, masks=None) -> Surface
Surface(size, flags=0, surface=...) -> Surface
Pygame object for representing images."#),
    (r#"surface"#, r#"SurfaceType"#, r#"Surface(size, flags=0, depth=0, masks=None) -> Surface
Surface(size, flags=0, surface=...) -> Surface
Pygame object for representing images."#),
    (r#"surfarray"#, r#"numpy_ndarray"#, r#"ndarray(shape, dtype=np.float64, buffer=None, offset=0, strides=None, order=None)

An array object represents a multidimensional, homogeneous array
of fixed-size items.  An associated data-type object describes the
format of each element in the array (its byte-order, how many bytes it
occupies in memory, whether it is an integer, a floating point number,
or something else, etc.)

Arrays should be constructed using `array`, `zeros` or `empty` (refer
to the See Also section below).  The parameters given here refer to
a low-level method (`ndarray(...)`) for instantiating an array.

For more information, refer to the `numpy` module and examine the
methods and attributes of an array.

Parameters
----------
(for the __new__ method; see Notes below)

shape : tuple of ints
    Shape of created array.
dtype : data-type, optional
    Any object that can be interpreted as a numpy data type.
    Default is `numpy.float64`.
buffer : object exposing buffer interface, optional
    Used to fill the array with data.
offset : int, optional
    Offset of array data in buffer.
strides : tuple of ints, optional
    Strides of data in memory.
order : {'C', 'F'}, optional
    Row-major (C-style) or column-major (Fortran-style) order.

Attributes
----------
T : ndarray
    Transpose of the array.
data : buffer
    The array's elements, in memory.
dtype : dtype object
    Describes the format of the elements in the array.
flags : dict
    Dictionary containing information related to memory use, e.g.,
    'C_CONTIGUOUS', 'OWNDATA', 'WRITEABLE', etc.
flat : numpy.flatiter object
    Flattened version of the array as an iterator.  The iterator
    allows assignments, e.g., ``x.flat = 3`` (See `ndarray.flat` for
    assignment examples; TODO).
imag : ndarray
    Imaginary part of the array.
real : ndarray
    Real part of the array.
size : int
    Number of elements in the array.
itemsize : int
    The memory use of each array element in bytes.
nbytes : int
    The total number of bytes required to store the array data,
    i.e., ``itemsize * size``.
ndim : int
    The array's number of dimensions.
shape : tuple of ints
    Shape of the array.
strides : tuple of ints
    The step-size required to move from one element to the next in
    memory. For example, a contiguous ``(3, 4)`` array of type
    ``int16`` in C-order has strides ``(8, 2)``.  This implies that
    to move from element to element in memory requires jumps of 2 bytes.
    To move from row-to-row, one needs to jump 8 bytes at a time
    (``2 * 4``).
ctypes : ctypes object
    Class containing properties of the array needed for interaction
    with ctypes.
base : ndarray
    If the array is a view into another array, that array is its `base`
    (unless that array is also a view).  The `base` array is where the
    array data is actually stored.

See Also
--------
array : Construct an array.
zeros : Create an array, each element of which is zero.
empty : Create an array, but leave its allocated memory unchanged (i.e.,
        it contains "garbage").
dtype : Create a data-type.
numpy.typing.NDArray : An ndarray alias :term:`generic <generic type>`
                       w.r.t. its `dtype.type <numpy.dtype.type>`.

Notes
-----
There are two modes of creating an array using ``__new__``:

1. If `buffer` is None, then only `shape`, `dtype`, and `order`
   are used.
2. If `buffer` is an object exposing the buffer interface, then
   all keywords are interpreted.

No ``__init__`` method is needed because the array is fully initialized
after the ``__new__`` method.

Examples
--------
These examples illustrate the low-level `ndarray` constructor.  Refer
to the `See Also` section above for easier ways of constructing an
ndarray.

First mode, `buffer` is None:

>>> import numpy as np
>>> np.ndarray(shape=(2,2), dtype=np.float64, order='F')
array([[0.0e+000, 0.0e+000], # random
       [     nan, 2.5e-323]])

Second mode:

>>> np.ndarray((2,), buffer=np.array([1,2,3]),
...            offset=np.int_().itemsize,
...            dtype=np.int_) # offset = 1*itemsize, i.e. skip first element
array([2, 3])"#),
    (r#"surfarray"#, r#"numpy_uint32"#, r#"Unsigned integer type, compatible with C ``unsigned long``.

:Character code: ``'L'``
:Canonical name: `numpy.ulong`
:Alias on this platform (win32 AMD64): `numpy.uint32`: 32-bit unsigned integer (``0`` to ``4_294_967_295``)."#),
    (r#"sysfont"#, r#"Font"#, r#"Font(filename=None, size=20) -> Font
Font(filename, size) -> Font
Font(pathlib.Path, size) -> Font
Font(object, size) -> Font
create a new Font object from a file"#),
    (r#"system"#, r#"PowerState"#, r#"PowerState(battery_percent: int | None, battery_seconds: int | None, on_battery: bool, no_battery: bool, charging: bool, charged: bool, plugged_in: bool, has_battery: bool)"#),
    (r#"time"#, r#"Clock"#, r#"Clock() -> Clock
Create an object to help track time."#),
    (r#"typing"#, r#"SequenceLike"#, r#"
Variant of the standard `Sequence` ABC that only requires `__getitem__` and `__len__`.
"#),
    (r#"version"#, r#"PygameVersion"#, r#"
Pygame Version class.
"#),
    (r#"version"#, r#"SDLVersion"#, r#"
SDL Version class.
"#),
    (r#"version"#, r#"SoftwareVersion"#, r#"
A class for storing data about software versions.
"#),
    (r#"window"#, r#"Window"#, r#"Window(title='pygame window', size=(640, 480), position=WINDOWPOS_UNDEFINED, *, fullscreen=..., fullscreen_desktop=..., opengl=..., vulkan=..., hidden=..., borderless=..., resizable=..., minimized=..., maximized=..., mouse_grabbed=..., keyboard_grabbed=..., input_focus=..., mouse_focus=..., allow_high_dpi=..., mouse_capture=..., always_on_top=..., utility=...) -> Window
Pygame object that represents a window."#),
];
