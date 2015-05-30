(ns chapter-tracker.model (:gen-class)
  (:use chapter-tracker.db)
  (:use chapter-tracker.model)
  (:require [clojure.java.jdbc :as sql])
)

(defn create-media [media-name base-dir file-types program]
  (println "SAVING" media-name)
  (try
    (wrap-connection
      (sql/insert-records :media_types {:name       media-name
                                        :base_dir   base-dir
                                        :file_types file-types
                                        :program    program})
    )
    true
    (catch Exception e
      (println "Unable to save" (str media-name \:))
      (println \tab (.getMessage e))
      false
    )
  )
)

(defrecord MediaRecord [media-id media-name base-dir file-types program]
  Object
  (toString [this] (:media-name this))
)

(defn fetch-media-record [id]
  (wrap-connection
    (sql/with-query-results rs ["SELECT * FROM media_types WHERE id = ?" id]
                            (if-let [row (first rs)]
                              (MediaRecord. (:id row)
                                             (:name row)
                                             (:base_dir row)
                                             (:file_types row)
                                             (:program row)
                              )
                            )
    )
  )
)

(defn fetch-media-records []
  (wrap-connection (sql/with-query-results rs ["SELECT * FROM media_types"]
                                           (doall (map #(MediaRecord. (:id %)
                                                                      (:name %)
                                                                      (:base_dir %)
                                                                      (:file_types %)
                                                                      (:program %)
                                                        ) rs))
                   )
  )
)

(defn create-series [media-type series-name episode-numbers-repeat-each-volume download-command-dir download-command]
  (println "SAVING" (format "%s(%s)" series-name media-type))
  (println (:media-id media-type))
  (try
    (wrap-connection
      (sql/insert-records :serieses {:media_type (:media-id media-type)
                                     :name       series-name
                                     :numbers_repeat_each_volume episode-numbers-repeat-each-volume
                                     :download_command_dir download-command-dir
                                     :download_command download-command})
    )
    true
    (catch Exception e
      (println "Unable to save" (str series-name \:))
      (println \tab (.getMessage e))
      false
    )
  )
)

(defrecord SeriesRecord [series-id media series-name episode-numbers-repeat-each-volume download-command-dir download-command]
  Object
  (toString [this] (format "%s(%s)" (:series-name this) (-> this :media :media-name)))
)

(defn fetch-series-record [id]
  (wrap-connection
    (sql/with-query-results rs ["SELECT * FROM serieses WHERE id = ?" id]
                            (if-let [row (first rs)]
                              (SeriesRecord. (:id row)
                                             (fetch-media-record (:media_type row))
                                             (:name row)
                                             (:numbers_repeat_each_volume row)
                                             (:download_command_dir row)
                                             (:download_command row)
                              )
                            )
    )
  )
)

(defn fetch-series-records []
  (wrap-connection
    (let [medias (apply hash-map (mapcat #(list (:media-id %) %) (fetch-media-records)))]
      (sql/with-query-results rs ["SELECT * FROM serieses"]
                              (doall (map #(SeriesRecord. (:id %)
                                                          (medias (:media_type %))
                                                          (:name %)
                                                          (:numbers_repeat_each_volume %)
                                                          (:download_command_dir %)
                                                          (:download_command %)
                                           ) rs))
      )
    )
  )
)

(defn update-series [series-id new-values-hash]
  (try
    (wrap-connection (sql/update-values :serieses
                                        ["id=?" series-id]
                                        new-values-hash
                     ))
    true
    (catch Exception e
      (println "Unable to update")
      (println \tab (.getMessage e))
      false
    )
  )
)

(defmulti delete-series-record number?)
(defmethod delete-series-record false [series-record]
  (delete-series-record (:series-id series-record))
)
(defmethod delete-series-record true [series-id]
  (wrap-connection (sql/delete-rows :serieses ["id=?" series-id])
                   (sql/delete-rows :directories ["series=?" series-id])
                   (sql/delete-rows :episodes ["series=?" series-id]))
)

(defn store-new-episodes [episodes]
  (wrap-connection (apply sql/insert-records :episodes episodes))
)

(defn all-files-for [series-id]
  (wrap-connection (sql/with-query-results rs ["SELECT file FROM episodes WHERE series=? AND NullIf(file,'') IS NOT NULL",series-id]
                                           (doall (set (map :file rs)))
                   ))
)

(defrecord DirectoryRecord [directory-id series directory pattern volume recursive])

(defn create-directory [series directory pattern volume recursive]
  (println "SAVING" directory)
  (try
    (wrap-connection
      (sql/insert-records :directories {:series    (:series-id series)
                                        :dir       directory
                                        :pattern   pattern
                                        :volume    volume
                                        :recursive recursive})
    )
    true
    (catch Exception e
      (println "Unable to save" (str directory \:))
      (println \tab (.getMessage e))
      false
    )
  )
)

(defmulti fetch-directory-records-for number?)
(defmethod fetch-directory-records-for true [series-id]
  (fetch-directory-records-for (fetch-series-record series-id))
)
(defmethod fetch-directory-records-for false [series]
  (wrap-connection (sql/with-query-results rs ["SELECT * FROM directories WHERE series=?",(:series-id series)]
                                           (doall (map #(DirectoryRecord. (:id %)
                                                                          series
                                                                          (:dir %)
                                                                          (:pattern %)
                                                                          (:volume %)
                                                                          (not= 0 (or (:recursive %) 0))
                                                        ) rs))
                   ))
)

(defn fetch-directory-record [directory-id]
  (wrap-connection (sql/with-query-results rs ["SELECT * FROM directories WHERE id=?",directory-id]
                     (if-let [first-in-rs (first rs)]
                       (DirectoryRecord. (:id first-in-rs)
                                         (-> first-in-rs :series fetch-series-record)
                                         (:dir first-in-rs)
                                         (:pattern first-in-rs)
                                         (:volume first-in-rs)
                                         (not= 0 (or (:recursive first-in-rs) 0))
                       ))))
)

(defn update-directory [directory-id new-values-hash]
  (try
    (wrap-connection (sql/update-values :directories
                                        ["id=?" directory-id]
                                        new-values-hash
                     ))
    true
    (catch Exception e
      (println "Unable to update")
      (println \tab (.getMessage e))
      false
    )
  )
)

(defmulti delete-directory-record number?)
(defmethod delete-directory-record false [directory-record]
  (delete-directory-record (:directory-id directory-record))
)
(defmethod delete-directory-record true [directory-id]
  (wrap-connection (sql/delete-rows :directories ["id=?" directory-id]))
)

(defrecord EpisodeRecord [episode-id series volume-number episode-number episode-name episode-file date-of-read]
  Object
  (toString [this] (str
                     (-> this :series .toString)
                     (if episode-number (str " episode " episode-number))
                     (if (:episode-name this) (str " - " (:episode-name this)) "")
                   )
  )
)

(defmulti fetch-episode-records-for number?)
(defmethod fetch-episode-records-for true [series-id]
  (fetch-episode-records-for (fetch-series-record series-id))
)
(defmethod fetch-episode-records-for false [series]
  (wrap-connection (sql/with-query-results rs ["SELECT * FROM episodes WHERE series=?",(:series-id series)]
                                           (doall (map #(EpisodeRecord. (:id %)
                                                                        series
                                                                        (:volume %)
                                                                        (:number %)
                                                                        (:name %)
                                                                        (:file %)
                                                                        (:date_of_read %)
                                                        ) rs))
                   ))
)

(defn fetch-episode-record [episode-id]
  (wrap-connection (sql/with-query-results rs ["SELECT * FROM episodes WHERE id=?",episode-id]
                                           (if-let [first-in-rs (first rs)]
                                             (EpisodeRecord. (:id first-in-rs)
                                                             (-> first-in-rs :series fetch-series-record)
                                                             (:volume first-in-rs)
                                                             (:number first-in-rs)
                                                             (:name first-in-rs)
                                                             (:file first-in-rs)
                                                             (:date_of_read first-in-rs)))
                   ))
)

(defn update-episode [episode-id new-values-hash]
  (try
    (wrap-connection (sql/update-values :episodes
                                        ["id=?" episode-id]
                                        new-values-hash
                     ))
    true
    (catch Exception e
      (println "Unable to update")
      (println \tab (.getMessage e))
      false
    )
  )
)

(defmulti delete-episode-record number?)
(defmethod delete-episode-record false [episode-record]
  (delete-episode-record (:episode-id episode-record)))
(defmethod delete-episode-record true [episode-id]
  (wrap-connection (sql/delete-rows :episodes ["id=?" episode-id])))
