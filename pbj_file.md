# PBJ File Format #

A very simple file format that has some resemblance to PLY but is easier
for prism to work with (at least, easier than with PLY files). I will write tools
that covnert regular the different file types to this file type. This is the only
file type that is supported by the scene description.

## Header ##

The header specifies the organization of data

 - p: position data in x, y, z form
 - 
 - f: face data is always triangulated, so it's coordinate in v0, v1, v2 form