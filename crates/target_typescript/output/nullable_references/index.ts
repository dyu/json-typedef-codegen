/**

 */

export interface Root {

    /**

     */

    notnull_ref_notnull_string: NotnullRefNotnullString;

    /**

     */

    notnull_ref_null_string: NotnullRefNullString;

    /**

     */

    notnull_string: NotnullString;

    /**

     */

    null_ref_notnull_string: NullRefNotnullString;

    /**

     */

    null_ref_null_string: NullRefNullString;

    /**

     */

    null_string: NullString;

}
/**

 */

export type NotnullRefNotnullString = NotnullString;
/**

 */

export type NotnullRefNullString = NullString;
/**

 */

export type NotnullString = string;
/**

 */

export type NullRefNotnullString = (NotnullString | null);
/**

 */

export type NullRefNullString = (NullString | null);
/**

 */

export type NullString = (string | null);