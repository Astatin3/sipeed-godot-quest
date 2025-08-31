# https://www.haiperformance.nl/de/blog/visualizing-point-clouds-in-godot-3/

class_name Pointcloud
extends MultiMeshInstance3D

var points: Array[Vector3] = []
var colors: Array[Vector3] = []

func _ready():
	multimesh = MultiMesh.new()
	multimesh.transform_format = MultiMesh.TRANSFORM_3D
	multimesh.use_colors=true
		
	var pmesh := PointMesh.new()
	var material := StandardMaterial3D.new()
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.albedo_color=Color(1,1,1)
	material.point_size=10
	material.vertex_color_use_as_albedo=true
	pmesh.material=material    
	
	multimesh.mesh=pmesh

func add_points(new_points:Array[Vector3],new_colors:Array[Vector3]):
	points.append_array(new_points)
	colors.append_array(new_colors)
	set_points(points, colors)

func set_points(points:Array[Vector3],colors:Array[Vector3]):
	multimesh.instance_count = len(points)
	
	for i in multimesh.instance_count:
		multimesh.set_instance_transform(i, Transform3D(Basis(), points[i]))
		multimesh.set_instance_color(i,Color(colors[i].x/256.0,colors[i].y/256.0,colors[i].z/256.0))
