import bpy
import sys

print("DEBUG: Script execution started")
print(f"DEBUG: sys.argv: {sys.argv}")

# Get command line arguments after "--"
argv = sys.argv
output_path = "assets/human_base.obj"
try:
    if "--" in argv:
        argv = argv[argv.index("--") + 1:]
        if len(argv) >= 1:
            output_path = argv[0]
except ValueError:
    pass

# List first 50 mesh objects to help selection
print("DEBUG: Listing MESH objects:")
meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
for i, obj in enumerate(meshes[:50]):
    print(f"DEBUG: MESH [{i}]: {obj.name}")

# Logic to select the best "Body"
candidates = [o for o in meshes if "body" in o.name.lower() and "male" in o.name.lower() and "stylized" not in o.name.lower()] 
if not candidates:
    candidates = [o for o in meshes if "body" in o.name.lower()]
if not candidates:
    candidates = meshes

selected = None
if candidates:
    for c in candidates:
        if "realistic" in c.name.lower() and "male" in c.name.lower():
             selected = c
             break
    if not selected:
        selected = candidates[0]

# Deselect all
bpy.ops.object.select_all(action='DESELECT')

if selected:
    print(f"DEBUG: Selected specific object: {selected.name}")
    selected.select_set(True)
    # Also select children? eyes/teeth might be separate?
    # For now just the body.
else:
    print("DEBUG: No suitable mesh found, selecting ALL")
    for obj in meshes:
        obj.select_set(True)

# Export to OBJ
exported = False
try:
    print("DEBUG: Attempting bpy.ops.wm.obj_export")
    bpy.ops.wm.obj_export(filepath=output_path, export_selected_objects=True)
    print(f"DEBUG: Exported using wm.obj_export to {output_path}")
    exported = True
except Exception as e:
    print(f"DEBUG: wm.obj_export failed: {e}")

if not exported:
    try:
        print("DEBUG: Attempting bpy.ops.export_scene.obj")
        bpy.ops.export_scene.obj(
            filepath=output_path,
            use_selection=True,
            use_materials=True,
            use_triangles=True, 
            axis_forward='-Z', 
            axis_up='Y'
        )
        print(f"DEBUG: Exported using export_scene.obj to {output_path}")
        exported = True
    except Exception as e:
        print(f"DEBUG: export_scene.obj failed: {e}")

if not exported:
    print("CRITICAL: All export methods failed")
    sys.exit(1)
