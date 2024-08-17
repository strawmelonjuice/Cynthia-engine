/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

export interface CynthiaPlugin {
  modifyResponseHTML?: (htmlin: string,
                        metadata: ContentMetaDataType,
                        Cynthia: CynthiaApiPoints) => string;
  modifyResponseHTMLBodyFragment?: (htmlin: string,
                                    metadata: ContentMetaDataType,
                                    Cynthia: CynthiaApiPoints) => string;
  modifyRequest?: (req: WebRequest,
                   Cynthia: CynthiaApiPoints) => void;
  onLoad?: (Cynthia: CynthiaApiPoints) => void;
  onClearInterval?: (Cynthia: CynthiaApiPoints) => void;
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
    meta: ContentMetaDataType;
    content: string;
  };
}
export interface ContentMetaDataType {
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
      author: undefined;
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
export type Responder = () => ResponderResponse | string;
/*
   * This is a simplified version of the Cynthia and CynthiaWebResponderApi classes.
   * It is used to pass the Cynthia object to the plugins, so they can use it to e.g. log messages to the console.
   * Currently, it's quite empty, but it will be expanded in the future.
 */
export class CynthiaApiPoints {
  public console: typeof terminalOut;
  constructor() {
    this.console = {
      log: terminalOut.log,
      error: terminalOut.error,
      warn: terminalOut.warn,
      info: terminalOut.info,
      debug: terminalOut.debug,
    };
  }
}
/*
   * This is a simplified version of the Cynthia and CynthiaWebResponderApi classes.
   * It is used to pass the Cynthia object to the plugins, so they can use it to e.g. log messages to the console.
   * Currently, it's quite empty, but it will be expanded in the future.
 */
export const CynthiaPassed =  new CynthiaApiPoints();

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
  // Method, URI, and headers are immutable
  // Metod is either GET or POST, no need to check for other methods, so it is fine being protected.
  protected readonly method: string;
  // URI is the path of the request, this might be matched with a regex, making it possible for
  // plugins to match multiple paths with one rule. It should as such not be protected.
  readonly uri: string;
  // Headers are the headers of the request, they are used to check for the presence of a header.
  // They can be read by the plugin here, or using the header method.
  readonly headers: Array<[string, string]>;
  // ID is the id of the request, it is used to identify the request in the response. It is immutable, and irrelevant to the plugin.
  private readonly id: number;
  // Once a request is claimed, it cannot be claimed again. This is how multiple plugins responding to the same request is handled.
  protected claimed: boolean;
  constructor(
      id: number,
      a: { method: string; uri: string; headers: Array<[string, string]> },
  ) {
    this.id = id;
    this.method = a.method;
    this.uri = a.uri;
    this.headers = a.headers;
    this.claimed = false;
  }
  // This method is used to get a header from the headers array. It returns the value of the header, or undefined if the header is not present.
  header(name: string) {
    return this.headers.find((header) => header[0] === name)?.[1];
  }
  protected respond(responder: Responder) {
    const responder_answ = (() => {
      const res = responder();
      if (typeof res === "string") {
        return {
          headers: [],
          body: res,
        };
      }
      return res;
    })();
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
  // Respond to a GET request, if the URI matches the request URI (wildcards supported), then the response in the callback is send back.
  get(adress: string, responder: Responder) {
    if (
        this.matchUris(this.uri, adress) &&
        this.method.toUpperCase() === "GET" &&
        !this.claimed
    ) {
      this.claimed = true;
      this.respond(responder);
    }
  }
  post(adress: string, responder: Responder) {
    if (
        this.matchUris(this.uri, adress) &&
        this.method.toUpperCase() === "POST" &&
        !this.claimed
    ) {
      this.claimed = true;
      this.respond(responder);
    }
  }
  escalate() {
    if (!this.claimed) {
      this.claimed = true;
      const response = new EmptyOKResponse(this.id);
      return Cynthia.send(response);
    }
    return;
  }
}
