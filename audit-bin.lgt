:- initialization((
	set_logtalk_flag(complements, allow),
	set_logtalk_flag(unknown_entities, silent),
	logtalk_load(lgtunit(loader)),
	logtalk_load('tests', [hook(lgtunit)]),
	(
		tests::run ->
		true
		write('msg(usr_input(true) == +1)'), nl
	;
	)
)).