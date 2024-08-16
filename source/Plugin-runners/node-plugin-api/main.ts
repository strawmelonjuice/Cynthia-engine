/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

export interface CynthiaPlugin {
  modifyResponseHTML?: (
    htmlin: string,
    Cynthia: typeof CynthiaPassed,
  ) => string;
  modifyResponseHTMLBodyFragment?: (
    htmlin: string,
    Cynthia: typeof CynthiaPassed,
  ) => string;
  modifyRequest?: (
    req: WebRequest,
    Cynthia: typeof CynthiaWebResponderApi,
  ) => void;
  onLoad?: () => void;
  onClearInterval?: () => void;
}

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

export interface EmptyOKResponseType {
  id: number;
  body: {
    as: "NoneOk";
  };
}
export class EmptyOKResponse implements EmptyOKResponseType {
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
    append_headers: Array<[string, string]>;
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
      | EmptyOKResponseType
      | OkStringResponseType
      | OkJSONResponse
      | ErrorResponse
      | WebResponse,
  ) => {
    console.log(`parse: ${JSON.stringify(res)}`);
  },
  console: terminalOut,
};
export interface ResponderResponse {
  headers: Array<[string, string]>;
  body: string;
}
export type Responder = () => ResponderResponse;

export class CynthiaWebResponderApi {
  cynthia_req_queue_id: number;
  constructor(id: number) {
    this.cynthia_req_queue_id = id;
  }
}

export const CynthiaPassed = {
  /*
   * This is a simplefied version of the Cynthia and CynthiaWebResponderApi classes.
   * It is used to pass the Cynthia object to the plugins, so they can use it to e.g. log messages to the console.
   * Currently, it's quite empty, but it will be expanded in the future.
   */
  console: terminalOut,
};

export interface IncomingWebRequest {
  id: number;
  body: {
    for: "WebRequest";
    method: string;
    uri: string;
    headers: Array<[string, string]>;
  };
}
export class WebRequest {
  private id: number;
  private method: string;
  uri: string;
  headers: Array<[string, string]>;
  private respondand: boolean;
  constructor(
    id: number,
    a: { method: string; uri: string; headers: Array<[string, string]> },
  ) {
    this.id = id;
    this.method = a.method;
    this.uri = a.uri;
    this.headers = a.headers;
    this.respondand = false;
  }
  private stillResponding() {
    return !this.respondand;
  }
  private respond(responder: Responder) {
    const responder_answ = responder();
    const response: WebResponse = {
      id: this.id,
      body: {
        as: "WebResponse",
        append_headers: responder_answ.headers,
        response_body: responder_answ.body,
      },
    };
    Cynthia.send(response);
  }
  private matchUris(str: string, rule: string) {
    if (str === rule) return true;

    // biome-ignore lint/style/noVar: This is a regex, not a variable
    var escapeRegex = (str: string) =>
      str.replace(/([.*+?^=!:${}()|\[\]\/\\])/g, "\\$1");
    return new RegExp(
      // biome-ignore lint/style/useTemplate: This is a regex, not a template
      "^" + rule.split("*").map(escapeRegex).join(".*") + "$",
    ).test(str);
  }

  get(adress: string, responder: Responder) {
    if (
      this.matchUris(this.uri, adress) &&
      this.method.toUpperCase() === "GET" &&
      this.stillResponding()
    ) {
      this.respondand = true;
      this.respond(responder);
    }
  }
  post(adress: string, responder: Responder) {
    if (
      this.matchUris(this.uri, adress) &&
      this.method.toUpperCase() === "POST" &&
      this.stillResponding()
    ) {
      this.respondand = true;
      this.respond(responder);
    }
  }
  escalate() {
    if (this.stillResponding()) {
      this.respondand = true;
      const response = new EmptyOKResponse(this.id);
      return Cynthia.send(response);
    }
    return;
  }
}
