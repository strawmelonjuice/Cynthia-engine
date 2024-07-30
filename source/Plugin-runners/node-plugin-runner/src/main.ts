/*
 * Copyright (c) 2024, MLC 'Strawmelonjuice' Bloeiman
 *
 * Licensed under the GNU AFFERO GENERAL PUBLIC LICENSE Version 3, see the LICENSE file for more information.
 */

const cynthia = {

}

const cynthiabase = {
    modifyOutputHTML: [
        (htmlin: string, cynthia) => {
            // Make no changes. Return unchanged.
            return htmlin;
        },
    ],
    modifyBodyHTML: [
        (htmlin: string) => {
            // Make no changes. Return unchanged.
            return htmlin;
        },
    ],
    requestOptions: [
        () => {
            void expressapp;
        },
    ],
    LogReader: [(type: string, msg: string) => { }],
};