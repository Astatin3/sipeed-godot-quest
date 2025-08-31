extends Node3D

const num_points: int = 50
const scale_pos: float = 0.01
const scale_color: float = 255. / num_points

@onready var right_hand: XRController3D = get_node("XROrigin3D/RightHand")

@onready var world_cloud: Pointcloud = get_node("World_Pointcloud")
@onready var hand_cloud: Pointcloud = get_node("XROrigin3D/RightHand/Hand_Pointcloud")

var points: Array[Vector3] = []
var colors: Array[Vector3] = []

var xr_interface: XRInterface

func _ready():
	xr_interface = XRServer.find_interface("OpenXR")
	if xr_interface and xr_interface.is_initialized():
		print("OpenXR initialized successfully")

		# Turn off v-sync!
		DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)

		# Change our main viewport to output to the HMD
		get_viewport().use_xr = true
		xr_interface.environment_blend_mode = XRInterface.XR_ENV_BLEND_MODE_ALPHA_BLEND
		get_viewport().transparent_bg = true
	else:
		print("OpenXR not initialized, please check if your headset is connected")
		
		
	
	for x in 50:
		for y in 50:
			points.append(Vector3(x * scale_pos, y * scale_pos, 0))
			colors.append(Vector3(x * scale_color, y * scale_color, 0))
	
	hand_cloud.set_points(points, colors)

var last_pressed = false

func _process(delta):
	var pressed = right_hand.get_float("trigger") > 0.
	if(pressed && !last_pressed):
		var points_clone: Array[Vector3] = []
		print(right_hand.global_position)
		for point in points:
			
			points_clone.append(right_hand.to_global(point))
		
		world_cloud.add_points(points_clone, colors.duplicate())
		last_pressed = true
		print("points! ", len(points_clone))
	elif (!pressed): last_pressed = false
