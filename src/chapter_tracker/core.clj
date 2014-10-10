(ns chapter-tracker.core (:gen-class)
  (:use chapter-tracker.view)
  (:use chapter-tracker.model)
  (:use chapter-tracker.schema)
  (:use chapter-tracker.file-matching)
)

(defn print-clj-trace [exception]
  (println (.toString exception))
  (doseq [trace-element (.getStackTrace exception) :when (.. trace-element getClassName (startsWith "chapter"))]
    (println \tab (.toString trace-element))
  )
)

(defn -main[]
  (try
    (create-tables)
    (catch Exception e))
  (try
    (show-frame)
    (catch Exception e
      (print-clj-trace e)
    )
  )
)

(defn -test[]
  ;(let [series (chapter-tracker.model/fetch-series-record 10)]
    ;(println series)
  ;)
  ;(chapter-tracker.file-matching/rescan-series (chapter-tracker.model/fetch-series-record 9))
  ;(let [ep (chapter-tracker.file-matching/guess-episode "kenichi v(?<v>\\d+) c(?<c>\\d+)" "kenichi v2 c8")]
    ;(println ep))
  ;(println "===")
  ;(let [ep (chapter-tracker.file-matching/guess-episode "kenichi" "hsdp kenichi  c1")]
    ;(println ep))
  ;(println (fetch-episode-record 20))
  ;(chapter-tracker.model/EpisodeRecord.)
  ;(println (-> (fetch-media-record 1) :program))
  ;(println (fetch-series-record 2))
  ;(.show (chapter-tracker.view.tools/create-delete-dialog "thingie" "thingus" #(println "hi")))
  ;(let [directory (chapter-tracker.model/fetch-directory-record 39)]
    ;(println "39 " (:volume directory))
    ;(println "39 " (:series directory))
  ;)
  ;(let [directory (chapter-tracker.model/fetch-directory-record 41)]
    ;(println "41 " (:volume directory))
  ;)
  ;(chapter-tracker.model/DirectoryRecord. 1 2 "hello" "hi")
  ;chapter-tracker.model/DirectoryRecord
  ;(println (find-new-files-for-series 22))
  ;(println (find-new-files-for-series 11))
)
