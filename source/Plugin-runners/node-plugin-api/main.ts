/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */


export interface Request {
    id: number
    body: TestRequestBody | unknown;
}
export interface GenericRequest {
    id: number
    body: {
        for: string;
    };
}

export interface TestRequest {
    id: number
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
            category?: string;
            author?: {
                name?: string;
                link?: string;
                thumbnail?: string;
            }
            dates: {
                altered: number;
                published: number;
            }
            thumbnail?: string;
        }
        content: string;
    }
}


let test: Request = {
    id: 0,
    "body": {
        for: "Tests",
        test: "Test"
    }
}

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
            as: "NoneOk"
        }
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
            value: value
        }
    }
}
export interface OkJSONResponse {
    id: number;
    body: {
        as: "OkJSON";
        value: unknown;
    };
}
export interface ErrorResponse {
    id: number;
    body: {
        as: "Error";
        message?: string | any;
    };
}
export class ErrorResponse implements ErrorResponse {
    id: number;
    body: { as: "Error"; message?: string | any };
    constructor(id: number, message?: string) {
        this.id = id;
        this.body = {
            as: "Error",
            message: (() =>{if (typeof message?.toString()) return message; else return "An error occurred."})()
        }
    }
}
export namespace terminalOut {
    export function log(str: unknown) {
        console.log(`log: ` + str);
    }
    export function error(str: unknown) {
        console.log(`error: ` + str);
    }
    export function warn(str: unknown) {
        console.log(`warn: ` + str);
    }
    export function info(str: unknown) {
        console.log(`info: ` + str);
    }
    export function debug(str: unknown) {
        console.log(`debug: ` + str);
    }
}
;

export const Cynthia = {
    send: (res: EmptyOKResponse | OkStringResponseType | OkJSONResponse | ErrorResponse) => {
        console.log(`parse: ${JSON.stringify(res)}`);
    },
    console: console
};
export namespace Incoming {
    export interface WebRequest {
        method: string;
        url: string;
        headers: Record<string, string>;
        body: string;
    }

}

