/*===========================================================================
*
*                            PUBLIC DOMAIN NOTICE
*               National Center for Biotechnology Information
*
*  This software/database is a "United States Government Work" under the
*  terms of the United States Copyright Act.  It was written as part of
*  the author's official duties as a United States Government employee and
*  thus cannot be copyrighted.  This software/database is freely available
*  to the public for use. The National Library of Medicine and the U.S.
*  Government have not placed any restriction on its use or reproduction.
*
*  Although all reasonable efforts have been taken to ensure the accuracy
*  and reliability of the software and data, the NLM and the U.S.
*  Government do not and cannot warrant the performance or results that
*  may be obtained by using this software or data. The NLM and the U.S.
*  Government disclaim all warranties, express or implied, including
*  warranties of performance, merchantability or fitness for any particular
*  purpose.
*
*  Please cite the author in any work or product based on this material.
*
* ===========================================================================
*
*/
#include <vdb/extern.h>

#include <sra/sradb.h>
#include <vdb/xform.h>
#include <klib/data-buffer.h>
#include <klib/text.h>
#include <klib/rc.h>
#include <sysalloc.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
#include <os-native.h>
#include <assert.h>

#include "name-tokenizer.h"

static
rc_t CC tokenize_spot_name_Illumina ( void *self, const VXformInfo *info, int64_t row_id,
                                   VRowResult *rslt, uint32_t argc, const VRowData argv [] )
{
    rc_t rc;
    const char *name, *end;
    spot_name_token_t *spot_name_tok;
    bool saw_end_float = false;
    const int EXPECTED_NUMBER_OF_TOKENS = 4;
    int tok = EXPECTED_NUMBER_OF_TOKENS;
    const uint16_t types[4] = {nt_L, nt_T, nt_X, nt_Y };
    
    assert(rslt->elem_bits == sizeof(spot_name_tok[0]) * 8);
    rslt->data->elem_bits = sizeof(spot_name_tok[0]) * 8;
    if( (rc = KDataBufferResize(rslt->data, EXPECTED_NUMBER_OF_TOKENS)) != 0 ) {
        return rc;
    }
    
    spot_name_tok = rslt->data->base;
    
    /* reverse line parse by format:
       /^(.+)[_:|]([0-9]+)[_:|]([0-9]+)[_:|]([+-0-9]+)[_:|]([+-0-9]+)([_:|][0-9]+?\.[0-9]+)?$/ = 
        (name, lane, tile, x, y, float?) = ($1, $2, $3, $4, $5, $6?)
    */
    name = &((const char *)argv[0].u.data.base)[argv[0].u.data.first_elem];
    end = name + argv[0].u.data.elem_count;
    while(rc == 0 && end > name && tok > 0 ) {
        const char* last_end = end--;
        bool not_numeric = false;
        char has_sign = 0;
        while( end >= name && strchr(":|_", *end) == NULL ) {
            if( !isdigit(*end) ) {
                not_numeric = true;
                if( (*end == '-' || *end == '+') && !has_sign ) {
                    has_sign = *end;
                    not_numeric = false;
                }
            }
            end--;
        }
        end++;
        if( not_numeric ) {
            if( !saw_end_float ) {
                /* skip optional trailing not integer number */
                saw_end_float = true;
            } else {
                rc = RC(rcSRA, rcFormatter, rcReading, rcName, rcInvalid);
            }
        } else {
            const char* c = end;
            if( *c == '-' ) {
                /* keep explicit - '-000' */
                if( last_end - c >= 2 && *(c + 1) == '0' ) {
                    c++;
                    while( *c == '0' && (c + 1) < last_end ) { c++; }
                    if( *c != '0' ) {
                        /* cannot tokenize -0004 */
                        break;
                    }
                }
            } else {
                if( *c == '+' ) {
                    /* keep explicit + */
                    c++;
                }
                /* keep leading zeroes */
                while(*c == '0' && (c + 1) < last_end ) { c++; }
            }
            tok--;
            spot_name_tok[tok].s.token_type = types[tok];
            spot_name_tok[tok].s.position = (uint16_t)(c - name);
            spot_name_tok[tok].s.length = (uint16_t)(last_end - c);
            if( spot_name_tok[tok].s.length == 0 ) {
                rc = RC(rcSRA, rcFormatter, rcReading, rcName, rcInvalid);
            }
        }
        /* back up to separator */
        end--;
    }
    if( rc == 0 && tok != 0 ) {
        rc = RC(rcSRA, rcFormatter, rcReading, rcName, rcInvalid);
    }
    if( rc != 0 ) {
        spot_name_tok[0].s.token_type = nt_unrecognized;
        spot_name_tok[0].s.position = 0;
        spot_name_tok[0].s.length = (uint16_t)argv[0].u.data.elem_count;
        rslt->elem_count = 1;
    } else {
        rslt->elem_count = EXPECTED_NUMBER_OF_TOKENS;
    }
    return 0;
}

/* tokenize_spot_name
 *  scans name on input
 *  tokenizes into parts

 extern function NCBI:SRA:spot_name_token
    NCBI:SRA:Illumina:tokenize_spot_name #1 ( ascii name );
 */
VTRANSFACT_IMPL ( NCBI_SRA_Illumina_tokenize_spot_name, 1, 0, 0 ) ( const void *self,
                  const VXfactInfo *info, VFuncDesc *rslt, const VFactoryParams *cp, const VFunctionParams *dp )
{
    rslt->variant = vftRow;
    rslt->u.rf = tokenize_spot_name_Illumina;
    
    return 0;
}
