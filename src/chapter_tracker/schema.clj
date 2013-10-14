(ns chapter-tracker.schema (:gen-class)
  (:use chapter-tracker.db)
  (:use chapter-tracker.table-definitions)
  (:require [clojure.java.jdbc :as sql])
)

(defn create-tables[]
  (wrap-connection
    (doseq [[table-name table-fields] tables]
      (if-let [existing-fields (try (sql/with-query-results rs [(format "PRAGMA table_info([%s])" (name table-name))]
             (doall (map :name rs))
                                   ) (catch java.sql.SQLException e nil))]
        ;When clause:
        (let [existing-fields-set (set existing-fields)]
          (doseq [field (filter #(-> % :field-name name existing-fields-set not) table-fields)]
            (sql/do-commands (format "ALTER TABLE [%s] ADD COLUMN %s %s %s;"
                             (name table-name)
                             (-> field :field-name name)
                             (-> field :field-type name)
                             (-> field :field-options (or "") name)
                     ))
          ))
        ;Else clause:
        (apply sql/create-table table-name [:id :integer "primary key" :autoincrement](map (fn [field]
                                                                                             (into [
                                                                                                    (field :field-name)
                                                                                                    (field :field-type)
                                                                                                   ](field :field-options))
                                                                                           ) table-fields))
      )
    )
    (try (sql/do-commands "CREATE UNIQUE INDEX serieses_unique ON serieses(media_type,name);") (catch Exception e))
  )
)
