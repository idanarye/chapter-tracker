(ns chapter-tracker.table-definitions (:gen-class))

(def tables (into {} (map
                       (fn [[t-name t-fields]]
                         [t-name (map (fn [[field-name field-type & field-options]] {
                                                                                     :field-name field-name
                                                                                     :field-type field-type
                                                                                     :field-options field-options
                                                                                    }) t-fields)]
                       )
                       {
                        :media_types [
                                      [:name :text :unique]
                                      [:base_dir :text]
                                      [:file_types :text]
                                      [:program :text]
                                     ]
                        :serieses    [
                                      [:media_type :integer]
                                      [:name :text]
                                     ]
                        :episodes    [
                                      [:series :integer]
                                      [:number :integer]
                                      [:name :text]
                                      [:file :text]
                                      [:date_of_read :datetime]
                                     ]
                        :directories [
                                      [:series :integer]
                                      [:pattern :text]
                                      [:dir :text]
                                     ]
                       })))
