/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

export interface Request {
	id: number;
	body: TestRequestBody | unknown;
}
export interface GenericRequest {
	id: number;
	body: {
		for: string;
	};
}

export interface TestRequest {
	id: number;
	body: TestRequestBody;
}
export interface TestRequestBody {
	for: "Test";
	test: string;
}
export interface ContentRenderRequest {
	id: number;
	body: ContentRenderRequestBody;
}

export interface ContentRenderRequestBody {
	for: "ContentRenderRequest";
	template_path: string;
	template_data: {
		meta: {
			id: string;
			title: string;
			desc?: string;
			tags: Array<string>;
			category?: string;
			author?: {
				name?: string;
				link?: string;
				thumbnail?: string;
			};
			dates: {
				altered: number;
				published: number;
			};
			thumbnail?: string;
		};
		content: string;
	};
}

export interface PostlistRenderRequest {
	id: number;
	body: PostlistRenderRequestBody;
}

export interface PostlistRenderRequestBody {
	for: "PostlistRenderRequest";
	template_path: string;
	template_data: {
		meta: {
			id: string;
			title: string;
			desc?: string;
			category?: string;
			tags: Array<string>;
			author: null;
			dates: {
				altered: number;
				published: number;
			};
			thumbnail?: string;
		};
		posts: Array<{
			id: string;
			title: string;
			short: string;
			dates: {
				altered: number;
				published: number;
			};
			thumbnail?: string;
			category: string;
			tags: Array<string>;
			author?: {
				name?: string;
				link?: string;
				thumbnail?: string;
			};
			postcontent: {
				Local: {
					source: {
						as: string;
						value: string;
					};
				};
			};
			scene_override: string;
		}>;
	};
}

let test: Request = {
	id: 0,
	body: {
		for: "Tests",
		test: "Test",
	},
};

export interface EmptyOKResponse {
	id: number;
	body: {
		as: "NoneOk";
	};
}
export class EmptyOKResponse implements EmptyOKResponse {
	body: { as: "NoneOk" };
	id: number;
	constructor(id: number) {
		this.id = id;
		this.body = {
			as: "NoneOk",
		};
	}
}
export interface OkStringResponseType {
	id: number;
	body: {
		as: "OkString";
		value: string;
	};
}

export class OkStringResponse implements OkStringResponseType {
	body: { as: "OkString"; value: string };
	id: number;
	constructor(id: number, value: string) {
		this.id = id;
		this.body = {
			as: "OkString",
			value: value,
		};
	}
}

export interface WebResponse {
	id: number;
	body: {
		as: "WebResponse";
		append_headers: Record<string, string>;
		response_body: string;
	};
}

export interface OkJSONResponse {
	id: number;
	body: {
		as: "OkJSON";
		value: unknown;
	};
}
export class ErrorResponse {
	id: number;
	body: { as: "Error"; message: string };
	constructor(id: number, message?: string | Error) {
		let msg = "An error occurred.";
		if (typeof message === "string") {
			msg = message;
		}
		if (message instanceof Error) {
			msg = message.message;
		}
		this.id = id;
		this.body = {
			as: "Error",
			message: msg,
		};
	}
}
export namespace terminalOut {
	export function log(str: unknown) {
		console.log(`log: ${str}`);
	}
	export function error(str: unknown) {
		console.log(`error: ${str}`);
	}
	export function warn(str: unknown) {
		console.log(`warn: ${str}`);
	}
	export function info(str: unknown) {
		console.log(`info: ${str}`);
	}
	export function debug(str: unknown) {
		console.log(`debug: ${str}`);
	}
}
export const Cynthia = {
	send: (
		res:
			| EmptyOKResponse
			| OkStringResponseType
			| OkJSONResponse
			| ErrorResponse
			| WebResponse
	) => {
		console.log(`parse: ${JSON.stringify(res)}`);
	},
	console: terminalOut,
};
export interface ResponderResponse {
	headers: Record<string, string>;
	body: string;
}
export type Responder = () => ResponderResponse;

export class CynthiaWebResponderApi {
	cynthia_req_queue_id: number;
	constructor(id: number) {
		this.cynthia_req_queue_id = id;
	}
	skip(to_request: IncomingWebRequest) {
		to_request.respondand = false;
		return to_request;
	}
	answer(to_request: IncomingWebRequest, answerrer: Responder) {
		const responder_answ = answerrer();
		const response: WebResponse = {
			id: this.cynthia_req_queue_id,
			body: {
				as: "WebResponse",
				append_headers: responder_answ.headers,
				response_body: responder_answ.body,
			},
		};
		Cynthia.send(response);
		to_request.respondand = true;
		return to_request;
	}
}

export const CynthiaPassed = {
	/*
	 * This is a simplefied version of the Cynthia and CynthiaWebResponderApi classes.
	 * It is used to pass the Cynthia object to the modifyOutputHTML and modifyBodyHTML
	 * (the string_passing_) functions.
	 * Currently, it's quite empty, but it will be expanded in the future.
	 */
};

export interface IncomingWebRequest {
	method: string;
	uri: string;
	headers: Record<string, string>;
	respondand: boolean;
}
