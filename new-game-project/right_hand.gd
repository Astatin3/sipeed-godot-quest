extends XRController3D

func _process(_t):
	var box: CSGBox3D = get_node("CSGBox3D2")
	var t = get_float("trigger") / 20
	box.size = Vector3(t,t,t)
