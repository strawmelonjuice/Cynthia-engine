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
        message?: string;
    };
}
export const Cynthia = {
    send: (res: EmptyOKResponse | OkStringResponseType | OkJSONResponse | ErrorResponse) => {
        console.log(`parse: ${JSON.stringify(res)}`);
    },
};
export namespace Incoming {
    export interface WebRequest {
        method: string;
        url: string;
        headers: Record<string, string>;
        body: string;
    }
}