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

#include "linker-priv.h"
#include "schema-parse.h"
#include "xform-priv.h"

#include <kfs/dyload.h>
#include <klib/token.h>
#include <klib/symtab.h>
#include <klib/symbol.h>
#include <klib/out.h>
#include <klib/rc.h>
#include <sysalloc.h>

#include <kfg/config.h>

#include <stdlib.h>
#include <string.h>
#include <byteswap.h>
#include <assert.h>

#include "transform-functions.h"

/* these functions need something to fill in VFuncDesc */
static
rc_t CC fake_stub_func ( void *self, const VXformInfo *info, int64_t row_id,
    VRowResult *rslt, uint32_t argc, const VRowData argv [] )
{
    assert(!"THIS FUNCTION IS NEVER TO BE CALLED");
    abort();
    return 0;
}

/* select is REALLY internal */
VTRANSFACT_BUILTIN_IMPL ( vdb_select, 1, 0, 0 ) ( const void *self,
    const VXfactInfo *info, VFuncDesc *rslt, const VFactoryParams *cp, const VFunctionParams *dp )
{
    /* set function pointer to non-NULL */
    rslt -> u . rf = fake_stub_func;
    rslt -> variant = vftSelect;
    return 0;
}

#if THIS_IS_A_STUB_FOR_COPYPASTA
/*
function < type T >
T passthru #1.0 ( T target )
     = vdb:stub:function;
 */
/* all pass-through functions are REALLY internal */
VTRANSFACT_BUILTIN_IMPL ( vdb_stub_function, 1, 0, 0 ) ( const void *self,
    const VXfactInfo *info, VFuncDesc *rslt, const VFactoryParams *cp, const VFunctionParams *dp )
{
    bool shouldPassThru = false;

    /* pass-through functions MUST have exactly one argument */
    assert(dp->argc == 1);

    /* This is where to implement logic to determine if pass-through is wanted
     */

    /* from here to the end of the function stays the same */
    if (shouldPassThru) {
        /* set function pointer to non-NULL */
        rslt -> u . rf = fake_stub_func;

        /* this is the magic that causes VFunctionProdRead to pass the Read
         * message on to the first argument of this function */
        rslt -> variant = vftPassThrough;
        return 0;
    }

    /* some non-zero RC to signal that pass-through is NOT supposed to happen
     * this will cause the schema to fall-through to the next rule or fail the
     * production
     */
    return SILENT_RC(rcVDB, rcFunction, rcConstructing, rcFunction, rcIgnored);
}
#endif

/* Pass-through function constructors should use this to prepare the result */
static rc_t maybePass(bool const shouldPass, VFuncDesc *const rslt)
{
    if (shouldPass) {
        rslt -> u . rf = fake_stub_func;
        rslt -> variant = vftPassThrough;
        return 0;
    }
    return SILENT_RC(rcVDB, rcFunction, rcConstructing, rcFunction, rcIgnored);
}

/* This function ALWAYS passes through
function < type T >
T passthru #1.0 ( T target )
     = vdb:passthru;
 */
/* all pass-through functions are REALLY internal */
VTRANSFACT_BUILTIN_IMPL ( vdb_passthru, 1, 0, 0 ) ( const void *self,
    const VXfactInfo *info, VFuncDesc *rslt, const VFactoryParams *cp, const VFunctionParams *dp )
{
    return maybePass(true, rslt);
}

static bool compare_node_value(KConfigNode const *node, unsigned const len, char const *const value)
{
    char buffer[4096];
    size_t num_read = 0, remaining = 0, offset = 0;

    do {
        KConfigNodeRead(node, offset, buffer, sizeof(buffer), &num_read, &remaining);
        if (offset + num_read + remaining != len)
            return false;
        assert(offset + num_read <= len);
        if (memcmp(buffer, value + offset, num_read) != 0)
            return false;
        offset += num_read;
    } while (num_read > 0 && remaining > 0);
    return offset == len;
}

static bool check_config_node(VFactoryParams const * const name_value)
{
    rc_t rc = 0;
    bool result = false;
    assert(name_value->argc == 2);
    {
        KConfig *cfg;
        rc = KConfigMake(&cfg, NULL);
        if (rc == 0) {
            KConfigNode const *node;
            rc = KConfigOpenNodeRead(cfg, &node, "%.*s"
                                     , (int)name_value->argv[0].count
                                     , name_value->argv[0].data.ascii);
            if (rc == 0) {
                result = compare_node_value(node
                                            , name_value->argv[1].count
                                            , name_value->argv[1].data.ascii);
                KConfigNodeRelease(node);
            }
            KConfigRelease(cfg);
        }
    }
    return result;
}

/*
function < type T >
T is_configuration_set #1.0 < ascii node, ascii value > ( T target )
     = vdb:is_configuration_set;
 */
/* all pass-through functions are REALLY internal */
VTRANSFACT_BUILTIN_IMPL ( vdb_is_configuration_set, 1, 0, 0 ) ( const void *self,
    const VXfactInfo *info, VFuncDesc *rslt, const VFactoryParams *cp, const VFunctionParams *dp )
{
    return maybePass(check_config_node(cp), rslt);
}

/* temporary silly stuff
 */

static
rc_t CC hello_func ( void *self, const VXformInfo *info, int64_t row_id,
    VRowResult *rslt, uint32_t argc, const VRowData argv [] )
{
    char *func_hello = self;
    OUTMSG (( "%s - row id %ld\n", func_hello, row_id ));
    return 0;
}

VTRANSFACT_BUILTIN_IMPL ( vdb_hello, 1, 0, 0 ) ( const void *self,
    const VXfactInfo *info, VFuncDesc *rslt, const VFactoryParams *cp, const VFunctionParams *dp )
{
    const char *fact_hello = "vdb:hello factory";
    const char *func_hello = "vdb:hello function";

    if ( cp -> argc > 0 )
    {
        fact_hello = cp -> argv [ 0 ] . data . ascii;
        if ( cp -> argc > 1 )
            func_hello = cp -> argv [ 1 ] . data . ascii;
    }

    rslt -> self = malloc ( strlen ( func_hello ) + 1 );
    if ( rslt -> self == NULL )
        return RC ( rcVDB, rcFunction, rcConstructing, rcMemory, rcExhausted );
    strcpy ( rslt -> self, func_hello );
    rslt -> whack = free;
    rslt -> u . rf = hello_func;
    rslt -> variant = vftRow;

    OUTMSG (( "%s - %u factory params, %u function params\n", fact_hello, cp -> argc, dp -> argc ));
    return 0;
}

/* InitFactories
 */
static
rc_t CC VLinkerEnterFactory ( KSymTable *tbl, const SchemaEnv *env,
    LFactory *lfact, const char *name )
{
    rc_t rc;

    KTokenSource src;
    KTokenText tt;
    KToken t;

    KTokenTextInitCString ( & tt, name, "VLinkerEnterFactory" );
    KTokenSourceInit ( & src, & tt );
    next_token ( tbl, & src, & t );

    rc = create_fqn ( tbl, & src, & t, env, ltFactory, lfact );
    if ( rc == 0 )
        lfact -> name = t . sym;

    return rc;
}

rc_t VLinkerAddFactories ( VLinker *self,
    const VLinkerIntFactory *fact, uint32_t count,
    KSymTable *tbl, const SchemaEnv *env )
{
    rc_t ret = 0;
    for ( uint32_t i = 0; i < count; ++ i )
    {
        LFactory *lfact = malloc ( sizeof * lfact );
        if ( lfact == NULL )
            return RC ( rcVDB, rcFunction, rcRegistering, rcMemory, rcExhausted );

        /* invoke factory to get description */
        rc_t rc = ( * fact [ i ] . f ) ( & lfact -> desc );
        if ( rc != 0 )
        {
            free ( lfact );
            return rc;
        }

        /* I am intrinsic and have no dl symbol */
        lfact -> addr = NULL;
        lfact -> name = NULL;
        lfact -> external = false;

        /* add to linker */
        rc = VectorAppend ( & self -> fact, & lfact -> id, lfact );
        if ( rc != 0 )
        {
            LFactoryWhack ( lfact, NULL );
            return rc;
        }

        /* create name */
        rc = VLinkerEnterFactory ( tbl, env, lfact, fact [ i ] . name );
        if ( rc != 0 )
        {
            void *ignore;
            VectorSwap ( & self -> fact, lfact -> id, NULL, & ignore );
            LFactoryWhack ( lfact, NULL );
            // if there is an earlier definition, ignore the new one
            if ( rc == SILENT_RC( rcVDB,rcSchema,rcParsing,rcToken,rcExists ) )
            {
                ret = rc;
            }
            else
            {
                return rc;
            }
        }
    }

    return ret;
}


static
rc_t CC VLinkerEnterSpecial ( KSymTable *tbl, const SchemaEnv *env,
    LSpecial *lspec, const char *name )
{
    rc_t rc;

    KTokenSource src;
    KTokenText tt;
    KToken t;

    KTokenTextInitCString ( & tt, name, "VLinkerEnterSpecial" );
    KTokenSourceInit ( & src, & tt );
    next_token ( tbl, & src, & t );

    rc = create_fqn ( tbl, & src, & t, env, ltUntyped, lspec );
    if ( rc == 0 )
        lspec -> name = t . sym;

    return rc;
}

static
rc_t VLinkerAddUntyped ( VLinker *self,
    const VLinkerIntSpecial *special, uint32_t count,
    KSymTable *tbl, const SchemaEnv *env )
{
    uint32_t i;
    for ( i = 0; i < count; ++ i )
    {
        rc_t rc;
        LSpecial *lspec = malloc ( sizeof * lspec );
        if ( lspec == NULL )
            return RC ( rcVDB, rcFunction, rcRegistering, rcMemory, rcExhausted );

        /* I am intrinsic and have no dl symbol */
        lspec -> addr = NULL;
        lspec -> name = NULL;
        lspec -> func = special [ i ] . f;

        /* add to linker */
        rc = VectorAppend ( & self -> special, & lspec -> id, lspec );
        if ( rc != 0 )
        {
            LSpecialWhack ( lspec, NULL );
            return rc;
        }

        /* create name */
        rc = VLinkerEnterSpecial ( tbl, env, lspec, special [ i ] . name );
        if ( rc != 0 )
        {
            void *ignore;
            VectorSwap ( & self -> special, lspec -> id, NULL, & ignore );
            LSpecialWhack ( lspec, NULL );
            return rc;
        }
    }

    return 0;
}

/* InitFactories
 */
rc_t VLinkerInitFactoriesRead ( VLinker *self,  KSymTable *tbl, const SchemaEnv *env )
{
    rc_t rc = VLinkerAddFactories ( self, fact, sizeof fact / sizeof fact [ 0 ], tbl, env );
    if ( rc == 0 )
        rc = VLinkerAddUntyped ( self, special, sizeof special / sizeof special [ 0 ], tbl, env );
    return rc;
}


/* MakeIntrinsic
 *  creates an initial, intrinsic linker
 *  pre-loaded with intrinsic factories
 */
rc_t VLinkerMakeIntrinsic ( VLinker **lp )
{
    KDyld *dl;
    rc_t rc = KDyldMake ( & dl );
    if ( rc == 0 )
    {
        rc = VLinkerMake ( lp, NULL, dl );
        KDyldRelease ( dl );

        if ( rc == 0 )
        {
            KSymTable tbl;
            VLinker *self = * lp;

            /* create symbol table with no intrinsic scope */
            rc = KSymTableInit ( & tbl, NULL );
            if ( rc == 0 )
            {
                SchemaEnv env;
                SchemaEnvInit ( & env, EXT_SCHEMA_LANG_VERSION );

                /* make intrinsic scope modifiable */
                KSymTablePushScope ( & tbl, & self -> scope );

                /* add intrinsic functions */
                rc = VLinkerInitFactories ( self, & tbl, & env );
                if ( rc == 0 )
                {
                    KSymTableWhack ( & tbl );
                    return 0;
                }

                KSymTableWhack ( & tbl );
            }

            VLinkerRelease ( self );
        }
    }

    * lp = NULL;

    return rc;
}
