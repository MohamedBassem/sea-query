use crate::{ConditionHolder, DynIden, IntoCondition, IntoIden, SimpleExpr};

#[derive(Debug, Clone, Default)]
pub struct OnConflict {
    pub(crate) target: Option<OnConflictTarget>,
    pub(crate) target_where: ConditionHolder,
    pub(crate) action: Option<OnConflictAction>,
    pub(crate) action_where: ConditionHolder,
}

/// Represents ON CONFLICT (upsert) targets
#[derive(Debug, Clone)]
pub enum OnConflictTarget {
    /// A list of columns with unique constraint
    ConflictColumns(Vec<DynIden>),
}

/// Represents ON CONFLICT (upsert) actions
#[derive(Debug, Clone)]
pub enum OnConflictAction {
    /// Do nothing
    DoNothing,
    /// Update column value of existing row
    Update(Vec<OnConflictUpdate>),
}

/// Represents strategies to update column in ON CONFLICT (upsert) actions
#[derive(Debug, Clone)]
pub enum OnConflictUpdate {
    /// Update column value of existing row with inserting value
    Column(DynIden),
    /// Update column value of existing row with expression
    Expr(DynIden, SimpleExpr),
}

impl OnConflict {
    /// Create a ON CONFLICT expression without target column,
    /// a special method designed for MySQL
    pub fn new() -> Self {
        Default::default()
    }

    /// Set ON CONFLICT target column
    pub fn column<C>(column: C) -> Self
    where
        C: IntoIden,
    {
        Self::columns([column])
    }

    /// Set ON CONFLICT target columns
    pub fn columns<I, C>(columns: I) -> Self
    where
        C: IntoIden,
        I: IntoIterator<Item = C>,
    {
        Self {
            target: Some(OnConflictTarget::ConflictColumns(
                columns.into_iter().map(IntoIden::into_iden).collect(),
            )),
            target_where: ConditionHolder::new(),
            action: None,
            action_where: ConditionHolder::new(),
        }
    }

    pub fn do_nothing(&mut self) -> &mut Self {
        self.action = Some(OnConflictAction::DoNothing);
        self
    }

    /// Set ON CONFLICT update column
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::insert()
    ///     .into_table(Glyph::Table)
    ///     .columns([Glyph::Aspect, Glyph::Image])
    ///     .values_panic([
    ///         "abcd".into(),
    ///         3.1415.into(),
    ///     ])
    ///     .on_conflict(
    ///         OnConflict::columns([Glyph::Id, Glyph::Aspect])
    ///             .update_column(Glyph::Aspect)
    ///             .value(Glyph::Image, Expr::val(1).add(2))
    ///             .to_owned()
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     [
    ///         r#"INSERT INTO `glyph` (`aspect`, `image`)"#,
    ///         r#"VALUES ('abcd', 3.1415)"#,
    ///         r#"ON DUPLICATE KEY UPDATE `aspect` = VALUES(`aspect`), `image` = 1 + 2"#,
    ///     ]
    ///     .join(" ")
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     [
    ///         r#"INSERT INTO "glyph" ("aspect", "image")"#,
    ///         r#"VALUES ('abcd', 3.1415)"#,
    ///         r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = 1 + 2"#,
    ///     ]
    ///     .join(" ")
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     [
    ///         r#"INSERT INTO "glyph" ("aspect", "image")"#,
    ///         r#"VALUES ('abcd', 3.1415)"#,
    ///         r#"ON CONFLICT ("id", "aspect") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = 1 + 2"#,
    ///     ]
    ///     .join(" ")
    /// );
    /// ```
    pub fn update_column<C>(&mut self, column: C) -> &mut Self
    where
        C: IntoIden,
    {
        self.update_columns([column])
    }

    /// Set ON CONFLICT update columns
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::insert()
    ///     .into_table(Glyph::Table)
    ///     .columns([Glyph::Aspect, Glyph::Image])
    ///     .values_panic([
    ///         2.into(),
    ///         3.into(),
    ///     ])
    ///     .on_conflict(
    ///         OnConflict::column(Glyph::Id)
    ///             .update_columns([Glyph::Aspect, Glyph::Image])
    ///             .to_owned(),
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"INSERT INTO `glyph` (`aspect`, `image`) VALUES (2, 3) ON DUPLICATE KEY UPDATE `aspect` = VALUES(`aspect`), `image` = VALUES(`image`)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = "excluded"."image""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") DO UPDATE SET "aspect" = "excluded"."aspect", "image" = "excluded"."image""#
    /// );
    /// ```
    pub fn update_columns<C, I>(&mut self, columns: I) -> &mut Self
    where
        C: IntoIden,
        I: IntoIterator<Item = C>,
    {
        let mut update_strats: Vec<OnConflictUpdate> = columns
            .into_iter()
            .map(|x| OnConflictUpdate::Column(IntoIden::into_iden(x)))
            .collect();

        match &mut self.action {
            Some(OnConflictAction::Update(v)) => {
                v.append(&mut update_strats);
            }
            Some(OnConflictAction::DoNothing) | None => {
                self.action = Some(OnConflictAction::Update(update_strats));
            }
        };
        self
    }

    /// Set ON CONFLICT update exprs
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::insert()
    ///     .into_table(Glyph::Table)
    ///     .columns([Glyph::Aspect, Glyph::Image])
    ///     .values_panic([
    ///         2.into(),
    ///         3.into(),
    ///     ])
    ///     .on_conflict(
    ///         OnConflict::column(Glyph::Id)
    ///             .value(Glyph::Image, Expr::val(1).add(2))
    ///             .to_owned()
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"INSERT INTO `glyph` (`aspect`, `image`) VALUES (2, 3) ON DUPLICATE KEY UPDATE `image` = 1 + 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") DO UPDATE SET "image" = 1 + 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") DO UPDATE SET "image" = 1 + 2"#
    /// );
    /// ```
    pub fn values<C, I>(&mut self, values: I) -> &mut Self
    where
        C: IntoIden,
        I: IntoIterator<Item = (C, SimpleExpr)>,
    {
        let mut update_exprs: Vec<OnConflictUpdate> = values
            .into_iter()
            .map(|(c, e)| OnConflictUpdate::Expr(c.into_iden(), e))
            .collect();

        match &mut self.action {
            Some(OnConflictAction::Update(v)) => {
                v.append(&mut update_exprs);
            }
            Some(OnConflictAction::DoNothing) | None => {
                self.action = Some(OnConflictAction::Update(update_exprs));
            }
        };
        self
    }

    /// Set ON CONFLICT update value
    pub fn value<C, T>(&mut self, col: C, value: T) -> &mut Self
    where
        C: IntoIden,
        T: Into<SimpleExpr>,
    {
        self.values([(col, value.into())])
    }

    /// Set target WHERE
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::insert()
    ///     .into_table(Glyph::Table)
    ///     .columns([Glyph::Aspect, Glyph::Image])
    ///     .values_panic([
    ///         2.into(),
    ///         3.into(),
    ///     ])
    ///     .on_conflict(
    ///         OnConflict::column(Glyph::Id)
    ///             .value(Glyph::Image, Expr::val(1).add(2))
    ///             .target_and_where(Expr::col((Glyph::Table, Glyph::Aspect)).is_null())
    ///             .to_owned()
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"INSERT INTO `glyph` (`aspect`, `image`) VALUES (2, 3) ON DUPLICATE KEY UPDATE `image` = 1 + 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") WHERE "glyph"."aspect" IS NULL DO UPDATE SET "image" = 1 + 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") WHERE "glyph"."aspect" IS NULL DO UPDATE SET "image" = 1 + 2"#
    /// );
    /// ```
    pub fn target_and_where(&mut self, other: SimpleExpr) -> &mut Self {
        self.target_cond_where(other)
    }

    /// Set target WHERE
    pub fn target_and_where_option(&mut self, other: Option<SimpleExpr>) -> &mut Self {
        if let Some(other) = other {
            self.target_cond_where(other);
        }
        self
    }

    /// Set target WHERE
    pub fn target_cond_where<C>(&mut self, condition: C) -> &mut Self
    where
        C: IntoCondition,
    {
        self.target_where.add_condition(condition.into_condition());
        self
    }

    /// Set action WHERE
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::insert()
    ///     .into_table(Glyph::Table)
    ///     .columns([Glyph::Aspect, Glyph::Image])
    ///     .values_panic([
    ///         2.into(),
    ///         3.into(),
    ///     ])
    ///     .on_conflict(
    ///         OnConflict::column(Glyph::Id)
    ///             .value(Glyph::Image, Expr::val(1).add(2))
    ///             .action_and_where(Expr::col((Glyph::Table, Glyph::Aspect)).is_null())
    ///             .to_owned()
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"INSERT INTO `glyph` (`aspect`, `image`) VALUES (2, 3) ON DUPLICATE KEY UPDATE `image` = 1 + 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") DO UPDATE SET "image" = 1 + 2 WHERE "glyph"."aspect" IS NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2, 3) ON CONFLICT ("id") DO UPDATE SET "image" = 1 + 2 WHERE "glyph"."aspect" IS NULL"#
    /// );
    /// ```
    pub fn action_and_where(&mut self, other: SimpleExpr) -> &mut Self {
        self.action_cond_where(other)
    }

    /// Set action WHERE
    pub fn action_and_where_option(&mut self, other: Option<SimpleExpr>) -> &mut Self {
        if let Some(other) = other {
            self.action_cond_where(other);
        }
        self
    }

    /// Set action WHERE
    pub fn action_cond_where<C>(&mut self, condition: C) -> &mut Self
    where
        C: IntoCondition,
    {
        self.action_where.add_condition(condition.into_condition());
        self
    }
}
