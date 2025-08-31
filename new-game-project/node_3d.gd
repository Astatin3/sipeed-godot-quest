extends Node3D

const num_points: int = 50
const scale_pos: float = -0.001
const scale_color: float = 1 / num_points

@onready var right_hand: XRController3D = get_node("XROrigin3D/RightHand")

@onready var world_cloud: Pointcloud = get_node("World_Pointcloud")
@onready var hand_cloud: Pointcloud = get_node("XROrigin3D/RightHand/Hand_Pointcloud")

var stream: Stream

var points: Array[Vector3] = []
var colors: Array[Color] = []

var xr_interface: XRInterface

var thread: Thread

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
		hand_cloud.global_scale(Vector3(scale_pos, scale_pos, scale_pos))
		#world_cloud.global_scale(Vector3(scale_pos, scale_pos, scale_pos))
	else:
		print("OpenXR not initialized, please check if your headset is connected")
		
		
	for x in 50:
		for y in 50:
			points.append(Vector3(x * 0.01, y * 0.01, 0))
			colors.append(Color(x * 0.01, y * 0.01, 0))
	
	hand_cloud.set_points(points, colors)
	
	stream = Stream.new()
	stream.start("10.42.0.1", 1234)


var last_pressed = false

func _process(delta):
	if(stream.new_points()):
		hand_cloud.set_points(stream.current_points, stream.current_colors)
	
	var pressed = right_hand.get_float("trigger") > 0.
	if(pressed && !last_pressed):
		var points_clone: Array[Vector3] = []
		print(right_hand.global_position)
		
		for point in stream.current_points:
			points_clone.append(right_hand.to_global(point * scale_pos))
		
		world_cloud.add_points(points_clone, stream.current_colors)
		last_pressed = true
		print("points! ", len(points_clone))
	elif (!pressed): last_pressed = false
	
	
