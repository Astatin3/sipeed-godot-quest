class_name Stream
extends Node

var thread: Thread
var _status: int = 0
var _stream: StreamPeerTCP = StreamPeerTCP.new()

func start(host: String, port: int):
	thread = Thread.new()
	thread.start(tcp_server.bind(host, port))

func tcp_server(host: String, port: int):
	while true:
		connect_to_host(host, port)
		
		while process():
			pass	
		
		OS.delay_msec(1000)

func connect_to_host(host: String, port: int) -> bool:
	print("Connecting to %s:%d" % [host, port])
	# Reset status so we can tell if it changes to error again.
	_status = _stream.STATUS_NONE
	if _stream.connect_to_host(host, port) != OK:
		print("Error connecting to host.")
		return false
	return true
		#emit_signal("error")

func process() -> bool:
	_stream.poll()
	var new_status: int = _stream.get_status()
	if new_status != _status:
		_status = new_status
		match _status:
			_stream.STATUS_NONE:
				print("Disconnected from host.")
				return false
				#emit_signal("disconnected")
			_stream.STATUS_CONNECTING:
				print("Connecting to host.")
				return true
			_stream.STATUS_CONNECTED:
				print("Connected to host.")
				#emit_signal("connected")
			_stream.STATUS_ERROR:
				print("Error with socket stream.")
				return false
				#emit_signal("error")

	if _status == _stream.STATUS_CONNECTED:
		parse_bytes()
		return true
		
	return false


var current_packet_length: int = -1
var current_index: int = 0

var current_points: Array[Vector3] = []
var current_colors: Array[Color] = []
var has_new_points = false

var overflow_bytes: Array = []


func parse_bytes():
	_stream.put_u8(2);
	
	var type = _stream.get_32()
	
	match type:
		1:
			var len = _stream.get_32()
			
			print("Got type %d with len %d" % [type, len])
			
			var points: Array[Vector3] = []
			var colors: Array[Color] = []
			#
			for i in len:
				points.append(Vector3(
					_stream.get_32(),
					_stream.get_32(),
					_stream.get_32()
				))
				#
				colors.append(Color(
					_stream.get_u8() / 256.,
					_stream.get_u8() / 256.,
					_stream.get_u8() / 256.
				))
			
			current_points = points
			current_colors = colors
			has_new_points = true


func new_points() -> bool:
	if(has_new_points):
		has_new_points = false
		return true
	return false
